// pallets/mev-shield/src/lib.rs
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

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
        AccountId32, DispatchErrorWithPostInfo, MultiSignature, RuntimeDebug,
        traits::{BadOrigin, Dispatchable, Hash, SaturatedConversion, Verify, Zero},
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

    /// Ephemeral key fingerprint used by off-chain code to verify the ML‑KEM pubkey.
    #[freeze_struct("4e13d24516013712")]
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct EphemeralPubKey {
        pub public_key: BoundedVec<u8, ConstU32<2048>>,
        pub epoch: u64,
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

    #[pallet::storage]
    pub type CurrentKey<T> = StorageValue<_, EphemeralPubKey, OptionQuery>;

    #[pallet::storage]
    pub type NextKey<T> = StorageValue<_, EphemeralPubKey, OptionQuery>;

    #[pallet::storage]
    pub type Epoch<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    pub type Submissions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash,
        Submission<T::AccountId, BlockNumberFor<T>, T::Hash>,
        OptionQuery,
    >;

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
    }

    #[pallet::error]
    pub enum Error<T> {
        BadEpoch,
        SubmissionAlreadyExists,
        MissingSubmission,
        CommitmentMismatch,
        SignatureInvalid,
        NonceMismatch,
        BadPublicKeyLen,
    }

    // ----------------- Hooks -----------------

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            if let Some(next) = <NextKey<T>>::take() {
                <CurrentKey<T>>::put(&next);
                <Epoch<T>>::mutate(|e| *e = next.epoch);
            }
            T::DbWeight::get().reads_writes(1, 2)
        }
    }

    // ----------------- Calls -----------------

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(
            Weight::from_parts(5_000, 0)
                .saturating_add(T::DbWeight::get().reads(0_u64))
                .saturating_add(T::DbWeight::get().writes(1_u64))
        )]
        pub fn announce_next_key(
            origin: OriginFor<T>,
            public_key: BoundedVec<u8, ConstU32<2048>>,
            epoch: u64,
        ) -> DispatchResult {
            // Only a current Aura validator may call this (signed account ∈ Aura authorities)
            T::AuthorityOrigin::ensure_validator(origin)?;

            const MAX_KYBER768_PK_LENGTH: usize = 1184;
            ensure!(
                public_key.len() == MAX_KYBER768_PK_LENGTH,
                Error::<T>::BadPublicKeyLen
            );

            NextKey::<T>::put(EphemeralPubKey {
                public_key: public_key.clone(),
                epoch,
            });

            Ok(())
        }

        /// Users submit an encrypted wrapper.
        ///
        /// `commitment` is `blake2_256(raw_payload)`, where:
        ///   raw_payload = signer || nonce || SCALE(call)
        ///
        /// `ciphertext` is constructed as:
        ///   [u16 kem_len] || kem_ct || nonce24 || aead_ct
        /// where:
        ///   - `kem_ct` is the ML‑KEM ciphertext (encapsulated shared secret)
        ///   - `aead_ct` is XChaCha20‑Poly1305 over:
        ///       signer || nonce || SCALE(call) || sig_kind || signature
        #[pallet::call_index(1)]
        #[pallet::weight(({
            let w = Weight::from_parts(ciphertext.len() as u64, 0)
                .saturating_add(T::DbWeight::get().reads(1_u64))
                .saturating_add(T::DbWeight::get().writes(1_u64));
            w
        }, DispatchClass::Normal, Pays::Yes))]
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

        /// Executed by the block author.
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(10_000, 0)
        .saturating_add(T::DbWeight::get().reads(3_u64))
        .saturating_add(T::DbWeight::get().writes(2_u64)))]
        pub fn execute_revealed(
            origin: OriginFor<T>,
            id: T::Hash,
            signer: T::AccountId,
            nonce: T::Nonce,
            call: Box<<T as Config>::RuntimeCall>,
            signature: MultiSignature,
        ) -> DispatchResultWithPostInfo {
            ensure_none(origin)?;

            let Some(sub) = Submissions::<T>::take(id) else {
                return Err(Error::<T>::MissingSubmission.into());
            };

            let payload_bytes = Self::build_raw_payload_bytes(&signer, nonce, call.as_ref());

            // 1) Commitment check against on-chain stored commitment.
            let recomputed: T::Hash = T::Hashing::hash(&payload_bytes);
            ensure!(sub.commitment == recomputed, Error::<T>::CommitmentMismatch);

            // 2) Signature check over the same payload.
            let genesis = frame_system::Pallet::<T>::block_hash(BlockNumberFor::<T>::zero());
            let mut msg = b"mev-shield:v1".to_vec();
            msg.extend_from_slice(genesis.as_ref());
            msg.extend_from_slice(&payload_bytes);
            ensure!(
                signature.verify(msg.as_slice(), &signer),
                Error::<T>::SignatureInvalid
            );

            // 3) Nonce check & bump.
            let acc = frame_system::Pallet::<T>::account_nonce(&signer);
            ensure!(acc == nonce, Error::<T>::NonceMismatch);
            frame_system::Pallet::<T>::inc_account_nonce(&signer);

            // 4) Dispatch inner call from signer.
            let info = call.get_dispatch_info();
            let required = info.call_weight.saturating_add(info.extension_weight);

            let origin_signed = frame_system::RawOrigin::Signed(signer.clone()).into();
            let res = (*call).dispatch(origin_signed);

            match res {
                Ok(post) => {
                    let actual = post.actual_weight.unwrap_or(required);
                    Self::deposit_event(Event::DecryptedExecuted { id, signer });
                    Ok(PostDispatchInfo {
                        actual_weight: Some(actual),
                        pays_fee: Pays::No,
                    })
                }
                Err(e) => {
                    Self::deposit_event(Event::DecryptedRejected { id, reason: e });
                    Ok(PostDispatchInfo {
                        actual_weight: Some(required),
                        pays_fee: Pays::No,
                    })
                }
            }
        }
    }

    impl<T: Config> Pallet<T> {
        /// Build the raw payload bytes used for both:
        ///   - `commitment = blake2_256(raw_payload)`
        ///   - signature message (after domain separation).
        ///
        /// Layout:
        ///   signer (32B) || nonce (u32 LE) || SCALE(call)
        fn build_raw_payload_bytes(
            signer: &T::AccountId,
            nonce: T::Nonce,
            call: &<T as Config>::RuntimeCall,
        ) -> Vec<u8> {
            let mut out = Vec::new();
            out.extend_from_slice(signer.as_ref());

            // We canonicalise nonce to u32 LE for the payload.
            let n_u32: u32 = nonce.saturated_into();
            out.extend_from_slice(&n_u32.to_le_bytes());

            // Append SCALE-encoded call.
            out.extend(call.encode());

            out
        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            match call {
                Call::execute_revealed { id, .. } => {
                    match source {
                        // Only allow locally-submitted / already-in-block txs.
                        TransactionSource::Local | TransactionSource::InBlock => {
                            ValidTransaction::with_tag_prefix("mev-shield-exec")
                                .priority(u64::MAX)
                                .longevity(64) // long because propagate(false)
                                .and_provides(id) // dedupe by wrapper id
                                .propagate(false) // CRITICAL: no gossip, stays on author node
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
