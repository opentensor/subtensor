use super::*;

use frame_benchmarking::v2::*;
use frame_support::{BoundedVec, pallet_prelude::ConstU32};
use frame_system::RawOrigin;
use sp_core::sr25519;
use sp_std::{vec, vec::Vec};

use chacha20poly1305::{
    KeyInit, XChaCha20Poly1305, XNonce,
    aead::{Aead, Payload},
};
use ml_kem::{
    Ciphertext, KemCore, MlKem768, MlKem768Params,
    kem::{Decapsulate, DecapsulationKey, Encapsulate},
};
use rand_chacha::{ChaCha20Rng, rand_core::SeedableRng};
use stp_shield::ShieldedTransaction;

use codec::Encode;
use sp_consensus_aura::AURA_ENGINE_ID;
use sp_core::crypto::KeyTypeId;
use sp_io::crypto::sr25519_generate;

/// Seed Aura authorities from sr25519 public keys.
fn seed_aura_authorities<T>(pubkeys: &[sr25519::Public])
where
    T: pallet::Config + pallet_aura::Config,
    <T as pallet_aura::Config>::AuthorityId: From<sr25519::Public>,
{
    pallet_aura::Authorities::<T>::mutate(|auths| {
        for pk in pubkeys {
            let auth_id: <T as pallet_aura::Config>::AuthorityId = (*pk).into();
            let _ = auths.try_push(auth_id);
        }
    });
}

/// Deposit an Aura pre-runtime digest for the given slot.
fn deposit_slot_digest<T: frame_system::Config>(slot: u64) {
    frame_system::Pallet::<T>::deposit_log(sp_runtime::DigestItem::PreRuntime(
        AURA_ENGINE_ID,
        slot.encode(),
    ));
}

/// Build a real max-size encrypted ciphertext (8192 bytes wire format).
///
/// Returns `(wire_ciphertext, dec_key)` so the benchmark can measure decryption.
fn build_max_encrypted_payload() -> (Vec<u8>, DecapsulationKey<MlKem768Params>) {
    let mut rng = ChaCha20Rng::from_seed([42u8; 32]);
    let (dec_key, enc_key) = MlKem768::generate(&mut rng);
    let (kem_ct, shared_secret) = enc_key.encapsulate(&mut rng).unwrap();

    // Wire overhead: key_hash(16) + kem_ct_len(2) + kem_ct(1088) + nonce(24) = 1130.
    // Max aead_ct = 8192 − 1130 = 7062.
    // Poly1305 tag = 16 bytes ⇒ max plaintext = 7046.
    let plaintext = vec![0x42u8; 7046];

    let nonce = [0u8; 24];
    let cipher = XChaCha20Poly1305::new(shared_secret.as_slice().into());
    let aead_ct = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: &plaintext,
                aad: &[],
            },
        )
        .expect("AEAD encryption must succeed in benchmark setup");

    let kem_ct_bytes = kem_ct.as_slice();
    let key_hash = [0u8; 16];

    let mut wire = Vec::with_capacity(8192);
    wire.extend_from_slice(&key_hash);
    wire.extend_from_slice(&(kem_ct_bytes.len() as u16).to_le_bytes());
    wire.extend_from_slice(kem_ct_bytes);
    wire.extend_from_slice(&nonce);
    wire.extend_from_slice(&aead_ct);

    debug_assert_eq!(wire.len(), 8192);

    (wire, dec_key)
}

#[benchmarks(
    where
        T: pallet_aura::Config,
        <T as pallet_aura::Config>::AuthorityId: From<sr25519::Public>,
)]
mod benches {
    use super::*;

    /// Worst-case `announce_next_key`: both current and next author exist,
    /// NextKey is populated (shift to CurrentKey), and the next author has a
    /// stored key (triggers NextKey write).
    #[benchmark]
    fn announce_next_key() {
        let alice = sr25519_generate(KeyTypeId(*b"aura"), Some("//Alice".as_bytes().to_vec()));
        let bob = sr25519_generate(KeyTypeId(*b"aura"), Some("//Bob".as_bytes().to_vec()));

        // Seed Aura with [alice, bob].
        seed_aura_authorities::<T>(&[alice, bob]);

        // Slot 0 → current = authorities[0 % 2] = alice,
        //          next    = authorities[1 % 2] = bob.
        deposit_slot_digest::<T>(0);

        // Pre-populate NextKey so the shift (CurrentKey ← NextKey) writes.
        let old_next_key: ShieldPublicKey = BoundedVec::truncate_from(vec![0x99; MLKEM768_PK_LEN]);
        NextKey::<T>::put(old_next_key);

        // Pre-populate AuthorKeys for the next author (bob) so NextKey gets set.
        let bob_key: ShieldPublicKey = BoundedVec::truncate_from(vec![0x77; MLKEM768_PK_LEN]);
        let bob_id: <T as pallet::Config>::AuthorityId = bob.into();
        AuthorKeys::<T>::insert(&bob_id, bob_key);

        // Valid 1184-byte ML-KEM-768 public key.
        let public_key: ShieldPublicKey = BoundedVec::truncate_from(vec![0x42; MLKEM768_PK_LEN]);

        #[extrinsic_call]
        announce_next_key(RawOrigin::None, Some(public_key.clone()));

        // CurrentKey was shifted from old NextKey.
        assert!(CurrentKey::<T>::get().is_some());
        // NextKey was set from bob's AuthorKeys entry.
        assert!(NextKey::<T>::get().is_some());
        // Alice's AuthorKeys was updated.
        let alice_id: <T as pallet::Config>::AuthorityId = alice.into();
        assert_eq!(AuthorKeys::<T>::get(&alice_id), Some(public_key));
    }

    /// Worst-case `submit_encrypted`: max-size ciphertext (8192 bytes) with
    /// real ML-KEM-768 + XChaCha20-Poly1305 decryption to account for the
    /// block proposer's off-chain decrypt cost.
    #[benchmark]
    fn submit_encrypted() {
        let who: T::AccountId = whitelisted_caller();

        // Build a real max-size encrypted payload.
        let (wire, dec_key) = build_max_encrypted_payload();
        let ciphertext: BoundedVec<u8, ConstU32<8192>> = BoundedVec::truncate_from(wire);

        #[block]
        {
            // 1. On-chain dispatch (event deposit).
            Pallet::<T>::submit_encrypted(
                RawOrigin::Signed(who.clone()).into(),
                ciphertext.clone(),
            )
            .expect("submit_encrypted dispatch must succeed");

            // 2. Parse wire-format ciphertext (proposer decode).
            let shielded_tx =
                ShieldedTransaction::parse(&ciphertext).expect("wire format must be valid");

            // 3. ML-KEM-768 decapsulate (proposer crypto).
            let ct = Ciphertext::<MlKem768>::try_from(shielded_tx.kem_ct.as_slice())
                .expect("kem_ct must be valid ML-KEM-768 ciphertext");
            let shared_secret = dec_key
                .decapsulate(&ct)
                .expect("decapsulation must succeed");
            let ss: [u8; 32] = shared_secret.into();

            // 4. AEAD decrypt (proposer crypto).
            let aead = XChaCha20Poly1305::new((&ss).into());
            let _plaintext = aead
                .decrypt(
                    XNonce::from_slice(&shielded_tx.nonce),
                    Payload {
                        msg: &shielded_tx.aead_ct,
                        aad: &[],
                    },
                )
                .expect("AEAD decryption must succeed");
        }
    }
}
