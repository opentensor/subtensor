// pallets/mev-shield/src/lib.rs
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
pub mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use codec::Encode;
    use frame_support::{
        dispatch::{GetDispatchInfo, PostDispatchInfo},
        pallet_prelude::*,
        traits::ConstU32,
        weights::Weight,
    };
    use frame_system::pallet_prelude::*;
    use sp_consensus_aura::sr25519::AuthorityId as AuraAuthorityId;
    use sp_core::ByteArray;
    use sp_runtime::transaction_validity::{
        InvalidTransaction, TransactionSource, ValidTransaction,
    };
    use sp_runtime::{
        AccountId32, DispatchErrorWithPostInfo, RuntimeDebug, Saturating,
        traits::{BadOrigin, Hash},
    };
    use sp_std::{marker::PhantomData, prelude::*};
    use subtensor_macros::freeze_struct;

    /// Origin helper: ensure the signer is an Aura authority (no session/authorship).
    pub struct EnsureAuraAuthority<T>(PhantomData<T>);

    pub trait AuthorityOriginExt<Origin> {
        type AccountId;

        fn ensure_validator(origin: Origin) -> Result<Self::AccountId, BadOrigin>;
    }

    impl<T> AuthorityOriginExt<OriginFor<T>> for EnsureAuraAuthority<T>
    where
        T: frame_system::Config<AccountId = AccountId32>
            + pallet_aura::Config<AuthorityId = AuraAuthorityId>,
    {
        type AccountId = AccountId32;

        fn ensure_validator(origin: OriginFor<T>) -> Result<Self::AccountId, BadOrigin> {
            let who: AccountId32 = frame_system::ensure_signed(origin)?;

            let aura_id =
                <AuraAuthorityId as ByteArray>::from_slice(who.as_ref()).map_err(|_| BadOrigin)?;

            let is_validator = pallet_aura::Authorities::<T>::get()
                .into_iter()
                .any(|id| id == aura_id);

            if is_validator {
                Ok(who)
            } else {
                Err(BadOrigin)
            }
        }
    }

    // ----------------- Types -----------------

    /// AEAD‑independent commitment over the revealed payload.
    #[freeze_struct("66e393c88124f360")]
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct Submission<AccountId, BlockNumber, Hash> {
        pub author: AccountId,
        pub commitment: Hash,
        pub ciphertext: BoundedVec<u8, ConstU32<8192>>,
        pub submitted_in: BlockNumber,
    }

    // ----------------- Config -----------------

    #[pallet::config]
    pub trait Config:
        frame_system::Config<AccountId = AccountId32, RuntimeEvent: From<Event<Self>>>
        + pallet_timestamp::Config
        + pallet_aura::Config
    {
        type RuntimeCall: Parameter
            + sp_runtime::traits::Dispatchable<
                RuntimeOrigin = Self::RuntimeOrigin,
                PostInfo = PostDispatchInfo,
            > + GetDispatchInfo;

        type AuthorityOrigin: AuthorityOriginExt<Self::RuntimeOrigin, AccountId = AccountId32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ----------------- Storage -----------------

    /// Current ML‑KEM‑768 public key bytes (encoded form).
    #[pallet::storage]
    pub type CurrentKey<T> = StorageValue<_, BoundedVec<u8, ConstU32<2048>>, OptionQuery>;

    /// Next ML‑KEM‑768 public key bytes, announced by the block author.
    #[pallet::storage]
    pub type NextKey<T> = StorageValue<_, BoundedVec<u8, ConstU32<2048>>, OptionQuery>;

    /// Buffered encrypted submissions, indexed by wrapper id.
    #[pallet::storage]
    pub type Submissions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash,
        Submission<T::AccountId, BlockNumberFor<T>, T::Hash>,
        OptionQuery,
    >;

    /// Hash(CurrentKey) per block, used to bind `key_hash` to the epoch at submit time.
    #[pallet::storage]
    pub type KeyHashByBlock<T: Config> =
        StorageMap<_, Blake2_128Concat, BlockNumberFor<T>, T::Hash, OptionQuery>;

    /// How many recent blocks of key-epoch hashes we retain.
    const KEY_EPOCH_HISTORY: u32 = 100;

    // ----------------- Events & Errors -----------------

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Encrypted wrapper accepted.
        EncryptedSubmitted { id: T::Hash, who: T::AccountId },
        /// Decrypted call executed.
        DecryptedExecuted { id: T::Hash, signer: T::AccountId },
        /// Decrypted execution rejected.
        DecryptedRejected {
            id: T::Hash,
            reason: DispatchErrorWithPostInfo<PostDispatchInfo>,
        },
        /// Decryption failed - validator could not decrypt the submission.
        DecryptionFailed {
            id: T::Hash,
            reason: BoundedVec<u8, ConstU32<256>>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// A submission with the same id already exists in `Submissions`.
        SubmissionAlreadyExists,
        /// The referenced submission id does not exist in `Submissions`.
        MissingSubmission,
        /// The recomputed commitment does not match the stored commitment.
        CommitmentMismatch,
        /// The provided signature over the payload is invalid.
        SignatureInvalid,
        /// The announced ML‑KEM public key length is invalid.
        BadPublicKeyLen,
        /// The MEV‑Shield key epoch for this submission has expired and is no longer accepted.
        KeyExpired,
        /// The provided `key_hash` does not match the expected epoch key hash.
        KeyHashMismatch,
    }

    // ----------------- Hooks -----------------

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            let db_weight = T::DbWeight::get();
            let mut reads: u64 = 0;
            let mut writes: u64 = 0;

            // 1) Roll NextKey -> CurrentKey if a next key is present.
            reads = reads.saturating_add(1);
            writes = writes.saturating_add(1);
            let mut current_opt: Option<BoundedVec<u8, ConstU32<2048>>> =
                if let Some(next) = NextKey::<T>::take() {
                    CurrentKey::<T>::put(&next);
                    writes = writes.saturating_add(1);
                    Some(next)
                } else {
                    None
                };

            // 2) If we didn't roll, read the existing CurrentKey exactly once.
            if current_opt.is_none() {
                reads = reads.saturating_add(1);
                current_opt = CurrentKey::<T>::get();
            }

            // 3) Maintain KeyHashByBlock entry for this block:
            match current_opt {
                Some(current) => {
                    let epoch_hash: T::Hash = T::Hashing::hash(current.as_ref());
                    KeyHashByBlock::<T>::insert(n, epoch_hash);
                    writes = writes.saturating_add(1);
                }
                None => {
                    KeyHashByBlock::<T>::remove(n);
                    writes = writes.saturating_add(1);
                }
            }

            // 4) Prune old epoch hashes with a sliding window of size KEY_EPOCH_HISTORY.
            let depth: BlockNumberFor<T> = KEY_EPOCH_HISTORY.into();
            if n >= depth {
                let prune_bn = n.saturating_sub(depth);
                KeyHashByBlock::<T>::remove(prune_bn);
                writes = writes.saturating_add(1);
            }

            // 5) TTL-based pruning of stale submissions.
            let ttl: BlockNumberFor<T> = KEY_EPOCH_HISTORY.into();
            let threshold: BlockNumberFor<T> = n.saturating_sub(ttl);

            let mut to_remove: Vec<T::Hash> = Vec::new();

            for (id, sub) in Submissions::<T>::iter() {
                reads = reads.saturating_add(1);
                if sub.submitted_in < threshold {
                    to_remove.push(id);
                }
            }

            for id in to_remove {
                Submissions::<T>::remove(id);
                writes = writes.saturating_add(1);
            }

            db_weight.reads_writes(reads, writes)
        }
    }

    // ----------------- Calls -----------------

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Announce the ML‑KEM public key that will become `CurrentKey` in
        /// the following block.
        #[pallet::call_index(0)]
        #[pallet::weight((
            Weight::from_parts(9_979_000, 0)
                .saturating_add(T::DbWeight::get().reads(1_u64))
                .saturating_add(T::DbWeight::get().writes(1_u64)),
            DispatchClass::Operational,
            Pays::No
        ))]
        #[allow(clippy::useless_conversion)]
        pub fn announce_next_key(
            origin: OriginFor<T>,
            public_key: BoundedVec<u8, ConstU32<2048>>,
        ) -> DispatchResult {
            ensure_none(origin)?;

            const MAX_KYBER768_PK_LENGTH: usize = 1184;
            ensure!(
                public_key.len() == MAX_KYBER768_PK_LENGTH,
                Error::<T>::BadPublicKeyLen
            );

            NextKey::<T>::put(public_key);

            Ok(())
        }

        /// Users submit an encrypted wrapper.
        ///
        /// Client‑side:
        ///
        ///   1. Read `NextKey` (ML‑KEM public key bytes) from storage.
        ///   2. Compute `key_hash = Hashing::hash(NextKey_bytes)`.
        ///   3. Build:
        ///
        ///        raw_payload = signer (32B AccountId)
        ///                    || key_hash (32B Hash)
        ///                    || SCALE(call)
        ///
        ///   4. `commitment = Hashing::hash(raw_payload)`.
        ///   5. Signature message:
        ///
        ///        "mev-shield:v1" || genesis_hash || raw_payload
        ///
        ///   6. Encrypt:
        ///
        ///        plaintext = raw_payload || sig_kind || signature(64B)
        ///
        ///      with ML‑KEM‑768 + XChaCha20‑Poly1305, producing
        ///
        ///        ciphertext = [u16 kem_len] || kem_ct || nonce24 || aead_ct
        ///
        #[pallet::call_index(1)]
        #[pallet::weight((
            Weight::from_parts(13_980_000, 0)
                .saturating_add(T::DbWeight::get().reads(1_u64))
                .saturating_add(T::DbWeight::get().writes(1_u64)),
            DispatchClass::Normal,
            Pays::Yes,
        ))]
        pub fn submit_encrypted(
            origin: OriginFor<T>,
            commitment: T::Hash,
            ciphertext: BoundedVec<u8, ConstU32<8192>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let id: T::Hash = T::Hashing::hash_of(&(who.clone(), commitment, &ciphertext));
            let sub = Submission::<T::AccountId, BlockNumberFor<T>, T::Hash> {
                author: who.clone(),
                commitment,
                ciphertext,
                submitted_in: <frame_system::Pallet<T>>::block_number(),
            };
            ensure!(
                !Submissions::<T>::contains_key(id),
                Error::<T>::SubmissionAlreadyExists
            );
            Submissions::<T>::insert(id, sub);
            Self::deposit_event(Event::EncryptedSubmitted { id, who });
            Ok(())
        }

        /// Marks a submission as failed to decrypt and removes it from storage.
        ///
        /// Called by the block author when decryption fails at any stage (e.g., ML-KEM decapsulate
        /// failed, AEAD decrypt failed, invalid ciphertext format, etc.). This allows clients to be
        /// notified of decryption failures through on-chain events.
        ///
        /// # Arguments
        ///
        /// * `id` - The wrapper id (hash of (author, commitment, ciphertext))
        /// * `reason` - Human-readable reason for the decryption failure (e.g., "ML-KEM decapsulate failed")
        #[pallet::call_index(3)]
        #[pallet::weight((
            Weight::from_parts(13_260_000, 0)
                .saturating_add(T::DbWeight::get().reads(1_u64))
                .saturating_add(T::DbWeight::get().writes(1_u64)),
            DispatchClass::Normal,
            Pays::No
        ))]
        pub fn mark_decryption_failed(
            origin: OriginFor<T>,
            id: T::Hash,
            reason: BoundedVec<u8, ConstU32<256>>,
        ) -> DispatchResultWithPostInfo {
            // Unsigned: only the author node may inject this via ValidateUnsigned.
            ensure_none(origin)?;

            // Load and consume the submission.
            let Some(_sub) = Submissions::<T>::take(id) else {
                return Err(Error::<T>::MissingSubmission.into());
            };

            // Emit event to notify clients
            Self::deposit_event(Event::DecryptionFailed { id, reason });

            Ok(().into())
        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            match call {
                Call::mark_decryption_failed { id, .. } => {
                    match source {
                        TransactionSource::Local | TransactionSource::InBlock => {
                            ValidTransaction::with_tag_prefix("mev-shield-failed")
                                .priority(1u64)
                                .longevity(64) // long because propagate(false)
                                .and_provides(id) // dedupe by wrapper id
                                .propagate(false) // CRITICAL: no gossip, stays on author node
                                .build()
                        }
                        _ => InvalidTransaction::Call.into(),
                    }
                }
                Call::announce_next_key { public_key, .. } => {
                    match source {
                        TransactionSource::Local | TransactionSource::InBlock => {
                            ValidTransaction::with_tag_prefix("mev-shield-failed")
                                .priority(10_000u64)
                                .longevity(4)
                                .and_provides(public_key) // dedupe by public_key
                                .propagate(true)
                                .build()
                        }
                        _ => InvalidTransaction::Call.into(),
                    }
                }
                _ => InvalidTransaction::Call.into(),
            }
        }
    }
}
