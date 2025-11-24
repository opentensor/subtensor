//! Benchmarking for pallet-mev-shield.
#![cfg(feature = "runtime-benchmarks")]

use super::*;

use codec::Encode;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

use frame_support::{BoundedVec, pallet_prelude::ConstU32};
use frame_system::pallet_prelude::BlockNumberFor;

use sp_core::crypto::KeyTypeId;
use sp_core::sr25519;
use sp_io::crypto::{sr25519_generate, sr25519_sign};

use sp_runtime::{
    AccountId32, MultiSignature,
    traits::{Hash as HashT, SaturatedConversion, Zero},
};

use sp_std::{boxed::Box, vec, vec::Vec};

/// Helper to build bounded bytes (public key) of a given length.
fn bounded_pk<const N: u32>(len: usize) -> BoundedVec<u8, ConstU32<N>> {
    let v = vec![7u8; len];
    BoundedVec::<u8, ConstU32<N>>::try_from(v).expect("within bound; qed")
}

/// Helper to build bounded bytes (ciphertext) of a given length.
fn bounded_ct<const N: u32>(len: usize) -> BoundedVec<u8, ConstU32<N>> {
    let v = vec![0u8; len];
    BoundedVec::<u8, ConstU32<N>>::try_from(v).expect("within bound; qed")
}

/// Build the raw payload bytes used by `commitment` & signature verification in the pallet.
/// Layout: signer (32B) || nonce (u32 LE) || SCALE(call)
fn build_payload_bytes<T: pallet::Config>(
    signer: &T::AccountId,
    nonce: <T as frame_system::Config>::Nonce,
    call: &<T as pallet::Config>::RuntimeCall,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(signer.as_ref());

    // canonicalize nonce to u32 LE
    let n_u32: u32 = nonce.saturated_into();
    out.extend_from_slice(&n_u32.to_le_bytes());

    // append SCALE-encoded call
    out.extend(call.encode());
    out
}

/// Seed Aura authorities so `EnsureAuraAuthority` passes for a given sr25519 pubkey.
///
/// We avoid requiring `ByteArray` on `AuthorityId` by relying on:
/// `<T as pallet_aura::Config>::AuthorityId: From<sr25519::Public>`.
fn seed_aura_authority_from_sr25519<T>(pubkey: &sr25519::Public)
where
    T: pallet::Config + pallet_aura::Config,
    <T as pallet_aura::Config>::AuthorityId: From<sr25519::Public>,
{
    let auth_id: <T as pallet_aura::Config>::AuthorityId = (*pubkey).into();
    pallet_aura::Authorities::<T>::mutate(|auths| {
        let _ = auths.try_push(auth_id);
    });
}

#[benchmarks(
    where
        // Needed to build a concrete inner call and convert into T::RuntimeCall.
        <T as pallet::Config>::RuntimeCall: From<frame_system::Call<T>>,
        // Needed so we can seed Authorities from a dev sr25519 pubkey.
        <T as pallet_aura::Config>::AuthorityId: From<sr25519::Public>,
)]
mod benches {
    use super::*;

    /// Benchmark `announce_next_key`.
    #[benchmark]
    fn announce_next_key() {
        // Generate a deterministic dev key in the host keystore (for benchmarks).
        // Any 4-byte KeyTypeId works for generation; it does not affect AccountId derivation.
        const KT: KeyTypeId = KeyTypeId(*b"benc");
        let alice_pub: sr25519::Public = sr25519_generate(KT, Some("//Alice".as_bytes().to_vec()));
        let alice_acc: AccountId32 = alice_pub.into();

        // Make this account an Aura authority for the generic runtime.
        seed_aura_authority_from_sr25519::<T>(&alice_pub);

        // Valid Kyber768 public key length per pallet check.
        const KYBER768_PK_LEN: usize = 1184;
        let public_key: BoundedVec<u8, ConstU32<2048>> = bounded_pk::<2048>(KYBER768_PK_LEN);
        let epoch: u64 = 42;

        // Measure: dispatch the extrinsic.
        #[extrinsic_call]
        announce_next_key(
            RawOrigin::Signed(alice_acc.clone()),
            public_key.clone(),
            epoch,
        );

        // Assert: NextKey should be set exactly.
        let stored = NextKey::<T>::get().expect("must be set by announce_next_key");
        assert_eq!(stored.epoch, epoch);
        assert_eq!(stored.public_key.as_slice(), public_key.as_slice());
    }

    /// Benchmark `submit_encrypted`.
    #[benchmark]
    fn submit_encrypted() {
        // Any whitelisted caller is fine (no authority requirement).
        let who: T::AccountId = whitelisted_caller();

        // Dummy commitment and ciphertext (bounded to 8192).
        let commitment: T::Hash = <T as frame_system::Config>::Hashing::hash(b"bench-commitment");
        const CT_DEFAULT_LEN: usize = 256;
        let ciphertext: BoundedVec<u8, ConstU32<8192>> = super::bounded_ct::<8192>(CT_DEFAULT_LEN);

        // Pre-compute expected id to assert postconditions.
        let id: T::Hash =
            <T as frame_system::Config>::Hashing::hash_of(&(who.clone(), commitment, &ciphertext));

        // Measure: dispatch the extrinsic.
        #[extrinsic_call]
        submit_encrypted(
            RawOrigin::Signed(who.clone()),
            commitment,
            ciphertext.clone(),
        );

        // Assert: stored under expected id.
        let got = Submissions::<T>::get(id).expect("submission must exist");
        assert_eq!(got.author, who);
        assert_eq!(
            got.commitment,
            <T as frame_system::Config>::Hashing::hash(b"bench-commitment")
        );
        assert_eq!(got.ciphertext.as_slice(), ciphertext.as_slice());
    }

    /// Benchmark `execute_revealed`.
    #[benchmark]
    fn execute_revealed() {
        // Generate a dev sr25519 key in the host keystore and derive the account.
        const KT: KeyTypeId = KeyTypeId(*b"benc");
        let signer_pub: sr25519::Public = sr25519_generate(KT, Some("//Alice".as_bytes().to_vec()));
        let signer: AccountId32 = signer_pub.into();

        // Inner call that will be executed as the signer (cheap & always available).
        let inner_call: <T as pallet::Config>::RuntimeCall = frame_system::Call::<T>::remark {
            remark: vec![1, 2, 3],
        }
        .into();

        // Nonce must match current system nonce (fresh account => 0).
        let nonce: <T as frame_system::Config>::Nonce = 0u32.into();

        // Build payload and commitment exactly how the pallet expects.
        let payload_bytes = super::build_payload_bytes::<T>(&signer, nonce, &inner_call);
        let commitment: <T as frame_system::Config>::Hash =
            <T as frame_system::Config>::Hashing::hash(payload_bytes.as_slice());

        // Ciphertext is stored in the submission but not used by `execute_revealed`; keep small.
        const CT_DEFAULT_LEN: usize = 64;
        let ciphertext: BoundedVec<u8, ConstU32<8192>> = super::bounded_ct::<8192>(CT_DEFAULT_LEN);

        // The submission `id` must match pallet's hashing scheme in submit_encrypted.
        let id: <T as frame_system::Config>::Hash = <T as frame_system::Config>::Hashing::hash_of(
            &(signer.clone(), commitment, &ciphertext),
        );

        // Seed the Submissions map with the expected entry.
        let sub = Submission::<T::AccountId, BlockNumberFor<T>, <T as frame_system::Config>::Hash> {
            author: signer.clone(),
            commitment,
            ciphertext: ciphertext.clone(),
            submitted_in: frame_system::Pallet::<T>::block_number(),
        };
        Submissions::<T>::insert(id, sub);

        // Domain-separated signing as in pallet: "mev-shield:v1" || genesis_hash || payload
        let zero: BlockNumberFor<T> = Zero::zero();
        let genesis = frame_system::Pallet::<T>::block_hash(zero);
        let mut msg = b"mev-shield:v1".to_vec();
        msg.extend_from_slice(genesis.as_ref());
        msg.extend_from_slice(&payload_bytes);

        // Sign using the host keystore and wrap into MultiSignature.
        let sig = sr25519_sign(KT, &signer_pub, &msg).expect("signing should succeed in benches");
        let signature: MultiSignature = sig.into();

        // Measure: dispatch the unsigned extrinsic (RawOrigin::None) with a valid wrapper.
        #[extrinsic_call]
        execute_revealed(
            RawOrigin::None,
            id,
            signer.clone(),
            nonce,
            Box::new(inner_call.clone()),
            signature.clone(),
        );

        // Assert: submission consumed, signer nonce bumped to 1.
        assert!(Submissions::<T>::get(id).is_none());
        let new_nonce = frame_system::Pallet::<T>::account_nonce(&signer);
        assert_eq!(new_nonce, 1u32.into());
    }
}
