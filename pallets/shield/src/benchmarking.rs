use super::*;

use frame_benchmarking::v2::*;
use frame_support::{BoundedVec, pallet_prelude::ConstU32};
use frame_system::{RawOrigin, pallet_prelude::BlockNumberFor};
use sp_core::{crypto::KeyTypeId, sr25519};
use sp_io::crypto::sr25519_generate;
use sp_runtime::{AccountId32, traits::Hash as HashT};
use sp_std::vec;

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

        // Measure: dispatch the extrinsic.
        #[extrinsic_call]
        announce_next_key(RawOrigin::Signed(alice_acc.clone()), public_key.clone());

        // Assert: NextKey should be set exactly.
        let stored = NextKey::<T>::get().expect("must be set by announce_next_key");
        assert_eq!(stored, public_key);
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

    /// Benchmark `mark_decryption_failed`.
    #[benchmark]
    fn mark_decryption_failed() {
        // Any account can be the author of the submission.
        let who: T::AccountId = whitelisted_caller();
        let submitted_in: BlockNumberFor<T> = frame_system::Pallet::<T>::block_number();

        // Build a dummy commitment and ciphertext.
        let commitment: T::Hash =
            <T as frame_system::Config>::Hashing::hash(b"bench-mark-decryption-failed");
        const CT_DEFAULT_LEN: usize = 32;
        let ciphertext: BoundedVec<u8, ConstU32<8192>> =
            BoundedVec::truncate_from(vec![0u8; CT_DEFAULT_LEN]);

        // Compute the submission id exactly like `submit_encrypted` does.
        let id: T::Hash =
            <T as frame_system::Config>::Hashing::hash_of(&(who.clone(), commitment, &ciphertext));

        // Seed Submissions with an entry for this id.
        let sub = Submission::<T::AccountId, BlockNumberFor<T>, <T as frame_system::Config>::Hash> {
            author: who,
            commitment,
            ciphertext: ciphertext.clone(),
            submitted_in,
        };
        Submissions::<T>::insert(id, sub);

        // Reason for failure.
        let reason: BoundedVec<u8, ConstU32<256>> =
            BoundedVec::truncate_from(b"benchmark-decryption-failed".to_vec());

        // Measure: dispatch the unsigned extrinsic.
        #[extrinsic_call]
        mark_decryption_failed(RawOrigin::None, id, reason);

        // Assert: submission is removed.
        assert!(Submissions::<T>::get(id).is_none());
    }
}
