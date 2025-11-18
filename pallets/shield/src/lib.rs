// pallets/mev-shield/src/lib.rs
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        dispatch::{GetDispatchInfo, PostDispatchInfo},
        pallet_prelude::*,
        traits::{ConstU32, Currency},
        weights::Weight,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::{
        AccountId32, MultiSignature, RuntimeDebug,
        traits::{BadOrigin, Dispatchable, Hash, Verify, Zero, SaturatedConversion},
        DispatchErrorWithPostInfo,
    };
    use sp_std::{marker::PhantomData, prelude::*};
    use subtensor_macros::freeze_struct;
    use sp_consensus_aura::sr25519::AuthorityId as AuraAuthorityId;
    use sp_core::ByteArray;
    use codec::Encode;

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
    #[freeze_struct("6c00690caddfeb78")]
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct Submission<AccountId, BlockNumber, Moment, Hash> {
        pub author: AccountId,
        pub key_epoch: u64,
        pub commitment: Hash,
        pub ciphertext: BoundedVec<u8, ConstU32<8192>>,
        pub payload_version: u16,
        pub submitted_in: BlockNumber,
        pub submitted_at: Moment,
        pub max_weight: Weight,
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
                PostInfo = PostDispatchInfo
            >
            + GetDispatchInfo;

        type AuthorityOrigin: AuthorityOriginExt<Self::RuntimeOrigin, AccountId = AccountId32>;

        #[pallet::constant]
        type SlotMs: Get<u64>;
        #[pallet::constant]
        type AnnounceAtMs: Get<u64>;
        #[pallet::constant]
        type GraceMs: Get<u64>;
        #[pallet::constant]
        type DecryptWindowMs: Get<u64>;

        type Currency: Currency<Self::AccountId>;
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
        Submission<T::AccountId, BlockNumberFor<T>, T::Moment, T::Hash>,
        OptionQuery,
    >;

    // ----------------- Events & Errors -----------------

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Encrypted wrapper accepted.
        EncryptedSubmitted { id: T::Hash, who: T::AccountId, epoch: u64 },
        /// Decrypted call executed.
        DecryptedExecuted { id: T::Hash, signer: T::AccountId },
        /// Decrypted execution rejected.
        DecryptedRejected { id: T::Hash, reason: DispatchErrorWithPostInfo<PostDispatchInfo> },
    }

    #[pallet::error]
    pub enum Error<T> {
        BadEpoch,
        SubmissionAlreadyExists,
        MissingSubmission,
        CommitmentMismatch,
        SignatureInvalid,
        WeightTooHigh,
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
            ensure!(public_key.len() == MAX_KYBER768_PK_LENGTH, Error::<T>::BadPublicKeyLen);

            NextKey::<T>::put(EphemeralPubKey { public_key: public_key.clone(), epoch });

            Ok(())
        }

        /// Users submit encrypted wrapper paying the normal fee.
        ///
        /// Commitment semantics:
        ///
        /// ```text
        /// raw_payload =
        ///   signer (32B) || nonce (u32 LE) || mortality_byte || SCALE(call)
        /// commitment = blake2_256(raw_payload)
        /// ```
        ///
        /// Ciphertext format: `[u16 kem_len][kem_ct][nonce24][aead_ct]`
        #[pallet::call_index(1)]
        #[pallet::weight({
            let w = Weight::from_parts(ciphertext.len() as u64, 0)
                .saturating_add(T::DbWeight::get().reads(1_u64))
                .saturating_add(T::DbWeight::get().writes(1_u64));
            w
        })]
        pub fn submit_encrypted(
            origin: OriginFor<T>,
            key_epoch: u64,
            commitment: T::Hash,
            ciphertext: BoundedVec<u8, ConstU32<8192>>,
            payload_version: u16,
            max_weight: Weight,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                key_epoch == Epoch::<T>::get() || key_epoch + 1 == Epoch::<T>::get(),
                Error::<T>::BadEpoch
            );

            let now = pallet_timestamp::Pallet::<T>::get();
            let id: T::Hash = T::Hashing::hash_of(&(who.clone(), commitment, &ciphertext));
            let sub = Submission::<T::AccountId, BlockNumberFor<T>, T::Moment, T::Hash> {
                author: who.clone(),
                key_epoch,
                commitment,
                ciphertext,
                payload_version,
                submitted_in: <frame_system::Pallet<T>>::block_number(),
                submitted_at: now,
                max_weight,
            };
            ensure!(
                !Submissions::<T>::contains_key(id),
                Error::<T>::SubmissionAlreadyExists
            );
            Submissions::<T>::insert(id, sub);
            Self::deposit_event(Event::EncryptedSubmitted {
                id,
                who,
                epoch: key_epoch,
            });
            Ok(())
        }

        /// Executed by the block author.
        #[pallet::call_index(2)]
        #[pallet::weight(
            Weight::from_parts(10_000, 0)
                .saturating_add(T::DbWeight::get().reads(3_u64))
                .saturating_add(T::DbWeight::get().writes(2_u64))
        )]
        pub fn execute_revealed(
            origin: OriginFor<T>,
            id: T::Hash,
            signer: T::AccountId,
            nonce: T::Nonce,
            mortality: sp_runtime::generic::Era,
            call: Box<<T as Config>::RuntimeCall>,
            signature: MultiSignature,
        ) -> DispatchResultWithPostInfo {
            ensure_none(origin)?;

            let Some(sub) = Submissions::<T>::take(id) else {
                return Err(Error::<T>::MissingSubmission.into());
            };

            let payload_bytes = Self::build_raw_payload_bytes(
                &signer,
                nonce,
                &mortality,
                call.as_ref(),
            );

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

            // 4) Dispatch inner call from signer; enforce max_weight guard.
            let info = call.get_dispatch_info();
            let required = info.call_weight.saturating_add(info.extension_weight);

            let leq = required.ref_time() <= sub.max_weight.ref_time()
                && required.proof_size() <= sub.max_weight.proof_size();
            ensure!(leq, Error::<T>::WeightTooHigh);

            let origin_signed = frame_system::RawOrigin::Signed(signer.clone()).into();
            let res = (*call).dispatch(origin_signed);

            match res {
                Ok(post) => {
                    let actual = post.actual_weight.unwrap_or(required);
                    Self::deposit_event(Event::DecryptedExecuted {
                        id,
                        signer,
                    });
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
        ///   signer (32B) || nonce (u32 LE) || mortality_byte || SCALE(call)
        fn build_raw_payload_bytes(
            signer: &T::AccountId,
            nonce: T::Nonce,
            mortality: &sp_runtime::generic::Era,
            call: &<T as Config>::RuntimeCall,
        ) -> Vec<u8> {
            let mut out = Vec::new();
            out.extend_from_slice(signer.as_ref());

            // We canonicalise nonce to u32 LE for the payload.
            let n_u32: u32 = nonce.saturated_into();
            out.extend_from_slice(&n_u32.to_le_bytes());

            // Simple 1-byte mortality code to match the off-chain layout.
            let m_byte: u8 = match mortality {
                sp_runtime::generic::Era::Immortal => 0,
                _ => 1,
            };
            out.push(m_byte);

            // Append SCALE-encoded call.
            out.extend(call.encode());

            out
        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(
            _source: sp_runtime::transaction_validity::TransactionSource,
            call: &Self::Call,
        ) -> sp_runtime::transaction_validity::TransactionValidity {
            use sp_runtime::transaction_validity::{
                InvalidTransaction,
                ValidTransaction,
            };

            match call {
                Call::execute_revealed { id, .. } => {
                    ValidTransaction::with_tag_prefix("mev-shield-exec")
                        .priority(u64::MAX)
                        .longevity(64)       // High because of propagate(false)
                        .and_provides(id)    // dedupe by wrapper id
                        .propagate(false)    // CRITICAL: no gossip, stays on author only
                        .build()
                }

                _ => InvalidTransaction::Call.into(),
            }
        }
    }
}
