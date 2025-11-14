// pallets/mev-shield/src/lib.rs
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        dispatch::{DispatchClass, GetDispatchInfo, Pays, PostDispatchInfo},
        pallet_prelude::*,
        traits::{ConstU32, Currency},
        weights::Weight,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::{
        AccountId32, MultiSignature, RuntimeDebug,
        traits::{Dispatchable, Hash, Verify, Zero, BadOrigin},
    };
    use sp_std::{marker::PhantomData, prelude::*};
    use subtensor_macros::freeze_struct;
    use sp_consensus_aura::sr25519::AuthorityId as AuraAuthorityId;
    use sp_core::ByteArray;

    // -------------------------------------------------------------------------
    // Origin helper: ensure the signer is an Aura authority (no session/authorship).
    // -------------------------------------------------------------------------
    //
    // This checks:
    //   1) origin is Signed(AccountId32)
    //   2) AccountId32 bytes map to an Aura AuthorityId
    //   3) that AuthorityId is a member of pallet_aura::Authorities
    //
    // NOTE: Assumes AccountId32 corresponds to sr25519 public key bytes (typical for Substrate).
/// Ensure that the origin is `Signed` by an account whose bytes map to the current
/// Aura `AuthorityId` and that this id is present in `pallet_aura::Authorities`.
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
        // Require a signed origin.
        let who: AccountId32 = frame_system::ensure_signed(origin)?;

        // Convert the raw 32 bytes of the AccountId into an Aura AuthorityId.
        // This uses `ByteArray::from_slice` to avoid any `sp_application_crypto` imports.
        let aura_id =
            <AuraAuthorityId as ByteArray>::from_slice(who.as_ref()).map_err(|_| BadOrigin)?;

        // Check the current authority set.
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
    /// We commit to `(signer, nonce, mortality, encoded_call)` (see `execute_revealed`).
    #[freeze_struct("62e25176827ab9b")]
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct Submission<AccountId, BlockNumber, Moment, Hash> {
        pub author: AccountId, // fee payer (wrapper submitter)
        pub key_epoch: u64,    // epoch for which this was encrypted
        pub commitment: Hash,  // chain hash over (signer, nonce, mortality, call)
        pub ciphertext: BoundedVec<u8, ConstU32<8192>>,
        pub payload_version: u16, // upgrade path
        pub submitted_in: BlockNumber,
        pub submitted_at: Moment,
        pub max_weight: Weight, // upper bound user is prepaying
    }

    /// Ephemeral key **fingerprint** used by off-chain code to verify the ML‑KEM pubkey it received via gossip.
    ///
    /// **Important:** `key` is **not** the ML‑KEM public key itself (those are ~1.1 KiB).
    /// We publish a 32‑byte `blake2_256(pk_bytes)` fingerprint instead to keep storage/events small.
    #[freeze_struct("daa971a48d20a3d9")]
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct EphemeralPubKey {
        /// Full Kyber768 public key bytes (length must be exactly 1184).
        pub public_key: BoundedVec<u8, ConstU32<2048>>,
        /// For traceability across announcements/rolls.
        pub epoch: u64,
    }

    // ----------------- Config -----------------

    #[pallet::config]
    pub trait Config:
        // System event type and AccountId32
        frame_system::Config<AccountId = AccountId32, RuntimeEvent: From<Event<Self>>>
        // Timestamp is used by the pallet.
        + pallet_timestamp::Config
        // 🔴 We read the Aura authority set (no session/authorship needed).
        + pallet_aura::Config
    {
        /// Allow dispatch of revealed inner calls.
        type RuntimeCall: Parameter
            + sp_runtime::traits::Dispatchable<
                RuntimeOrigin = Self::RuntimeOrigin,
                PostInfo = PostDispatchInfo
            >
            + GetDispatchInfo;

        /// Who may announce the next ephemeral key.
        ///
        /// In your runtime set:
        ///     type AuthorityOrigin = pallet_mev_shield::EnsureAuraAuthority<Self>;
        ///
        /// This ensures the signer is a current Aura authority.
        type AuthorityOrigin: AuthorityOriginExt<Self::RuntimeOrigin, AccountId = AccountId32>;

        /// Parameters exposed on-chain for light clients / UI (ms).
        #[pallet::constant]
        type SlotMs: Get<u64>;
        #[pallet::constant]
        type AnnounceAtMs: Get<u64>; // e.g., 7000
        #[pallet::constant]
        type GraceMs: Get<u64>; // e.g., 2000 (old key valid until 9s)
        #[pallet::constant]
        type DecryptWindowMs: Get<u64>; // e.g., 3000 (last 3s)

        /// Currency for fees (wrapper pays normal tx fee via regular machinery).
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

    /// All encrypted submissions live here until executed or discarded.
    #[pallet::storage]
    pub type Submissions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash, // id = hash(author, commitment, ciphertext)
        Submission<T::AccountId, BlockNumberFor<T>, T::Moment, T::Hash>,
        OptionQuery,
    >;

    /// Mark a submission id as consumed (executed or invalidated).
    #[pallet::storage]
    pub type Consumed<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, (), OptionQuery>;

    // ----------------- Events & Errors -----------------

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Next ML‑KEM public key bytes announced.
        NextKeyAnnounced {
            public_key: Vec<u8>, // full Kyber768 public key (1184 bytes)
            epoch: u64,
            at_ms: u64,
        },
        /// Current key rolled to the next (happens on_initialize of new block).
        KeyRolled {
            public_key: Vec<u8>, // full Kyber768 public key (1184 bytes)
            epoch: u64
        },
        /// Encrypted wrapper accepted.
        EncryptedSubmitted { id: T::Hash, who: T::AccountId, epoch: u64 },
        /// Decrypted call executed.
        DecryptedExecuted { id: T::Hash, signer: T::AccountId, actual_weight: Weight },
        /// Decrypted execution rejected (mismatch, overweight, bad sig, etc.).
        DecryptedRejected { id: T::Hash, reason: u8 },
    }

    #[pallet::error]
    pub enum Error<T> {
        BadEpoch,
        AlreadyConsumed,
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
        /// Roll the keys once per block (current <= next).
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            if let Some(next) = <NextKey<T>>::take() {
                <CurrentKey<T>>::put(&next);
                <Epoch<T>>::mutate(|e| *e = next.epoch);

                // Emit event with the full public key bytes (convert BoundedVec -> Vec for the event).
                Self::deposit_event(Event::<T>::KeyRolled {
                    public_key: next.public_key.to_vec(),
                    epoch: next.epoch,
                });
            }
            // small constant cost
            T::DbWeight::get().reads_writes(1, 2)
        }
    }

    // ----------------- Calls -----------------

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Validators announce the *next* ephemeral **ML‑KEM** public key bytes.
        /// Origin is restricted to the PoA validator set (Aura authorities).
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
            at_ms: u64,
        ) -> DispatchResult {
            // ✅ Only a current Aura validator may call this (signed account ∈ Aura authorities)
            T::AuthorityOrigin::ensure_validator(origin)?;

            // Enforce Kyber768 pk length (1184 bytes).
            ensure!(public_key.len() == 1184, Error::<T>::BadPublicKeyLen);

            NextKey::<T>::put(EphemeralPubKey { public_key: public_key.clone(), epoch });

            // Emit full bytes in the event (convert to Vec for simplicity).
            Self::deposit_event(Event::NextKeyAnnounced {
                public_key: public_key.to_vec(),
                epoch,
                at_ms,
            });
            Ok(())
        }

        /// Users submit encrypted wrapper paying the normal fee.
        /// `commitment = blake2_256( SCALE( (signer, nonce, mortality, call) ) )`
        ///
        /// Ciphertext format (see module docs): `[u16 kem_len][kem_ct][nonce24][aead_ct]`
        #[pallet::call_index(1)]
        #[pallet::weight({
            let w = Weight::from_parts(ciphertext.len() as u64, 0)
                .saturating_add(T::DbWeight::get().reads(1_u64))   // Epoch::get
                .saturating_add(T::DbWeight::get().writes(1_u64)); // Submissions::insert
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
                Error::<T>::AlreadyConsumed
            );
            Submissions::<T>::insert(id, sub);
            Self::deposit_event(Event::EncryptedSubmitted {
                id,
                who,
                epoch: key_epoch,
            });
            Ok(())
        }

        /// Executed by the block author (unsigned) in the last ~3s.
        /// The caller provides the plaintext (signed) and we:
        ///  - check commitment
        ///  - check signature
        ///  - check nonce (and increment)
        ///  - ensure weight <= max_weight
        ///  - dispatch the call as the signer (fee-free here; fee already paid by wrapper)
        #[pallet::call_index(2)]
        #[pallet::weight(
            Weight::from_parts(10_000, 0)
                .saturating_add(T::DbWeight::get().reads(3_u64))
                .saturating_add(T::DbWeight::get().writes(2_u64))
        )]
        pub fn execute_revealed(
            origin: OriginFor<T>,
            id: T::Hash,
            signer: T::AccountId, // AccountId32 due to Config bound
            nonce: T::Nonce,
            mortality: sp_runtime::generic::Era, // same type used in signed extrinsics
            call: Box<<T as Config>::RuntimeCall>,
            signature: MultiSignature,
        ) -> DispatchResultWithPostInfo {
            ensure_none(origin)?;
            ensure!(
                !Consumed::<T>::contains_key(id),
                Error::<T>::AlreadyConsumed
            );
            let Some(sub) = Submissions::<T>::take(id) else {
                return Err(Error::<T>::MissingSubmission.into());
            };

            // 1) Commitment check (encode by-ref to avoid Clone bound on RuntimeCall)
            let payload_bytes = (signer.clone(), nonce, mortality.clone(), call.as_ref()).encode();
            let recomputed: T::Hash = T::Hashing::hash_of(&payload_bytes);
            ensure!(sub.commitment == recomputed, Error::<T>::CommitmentMismatch);

            // 2) Signature check over the same payload (domain separated)
            let genesis = frame_system::Pallet::<T>::block_hash(BlockNumberFor::<T>::zero());
            let mut msg = b"mev-shield:v1".to_vec();
            msg.extend_from_slice(genesis.as_ref());
            msg.extend_from_slice(&payload_bytes);
            ensure!(
                signature.verify(msg.as_slice(), &signer),
                Error::<T>::SignatureInvalid
            );

            // 3) Nonce check & bump (we mimic system signed extension behavior)
            let acc = frame_system::Pallet::<T>::account_nonce(&signer);
            ensure!(acc == nonce, Error::<T>::NonceMismatch);
            frame_system::Pallet::<T>::inc_account_nonce(&signer);

            // 4) Dispatch inner call from signer; enforce max_weight guard
            let info = call.get_dispatch_info();
            let required = info.call_weight.saturating_add(info.extension_weight);

            let leq = required.ref_time() <= sub.max_weight.ref_time()
                && required.proof_size() <= sub.max_weight.proof_size();
            ensure!(leq, Error::<T>::WeightTooHigh);

            let origin_signed = frame_system::RawOrigin::Signed(signer.clone()).into();
            let res = (*call).dispatch(origin_signed);

            // Mark as consumed regardless of outcome (user already paid wrapper fee)
            Consumed::<T>::insert(id, ());

            match res {
                Ok(post) => {
                    let actual = post.actual_weight.unwrap_or(required);
                    Self::deposit_event(Event::DecryptedExecuted {
                        id,
                        signer,
                        actual_weight: actual,
                    });
                    Ok(PostDispatchInfo {
                        actual_weight: Some(actual),
                        pays_fee: Pays::No,
                    })
                }
                Err(_e) => {
                    Self::deposit_event(Event::DecryptedRejected { id, reason: 1 });
                    Ok(PostDispatchInfo {
                        actual_weight: Some(required),
                        pays_fee: Pays::No,
                    })
                }
            }
        }
    }


        #[pallet::validate_unsigned]
        impl<T: Config> ValidateUnsigned for Pallet<T> {
            type Call = Call<T>;

            fn validate_unsigned(
                _source: sp_runtime::transaction_validity::TransactionSource,
                call: &Self::Call,
            ) -> sp_runtime::transaction_validity::TransactionValidity {
                use sp_runtime::transaction_validity::{InvalidTransaction, ValidTransaction};

                match call {
                    // This is the only unsigned entry point.
                    Call::execute_revealed { id, .. } => {
                        // Mark this unsigned tx as valid for the pool & block builder.
                        //
                        // IMPORTANT:
                        //  - We *do* want it in the local pool so that the block author
                        //    can include it in the next block.
                        //  - We *do not* want it to be gossiped, otherwise the cleartext
                        //    MEV‑shielded call leaks to the network.
                        //
                        // `propagate(false)` keeps it strictly local.
                        ValidTransaction::with_tag_prefix("mev-shield-exec")
                            .priority(u64::MAX) // always prefer executes when present
                            .longevity(1)       // only for the very next block
                            .and_provides(id)   // crucial: at least one tag
                            .propagate(false)   // 👈 no gossip / no mempool MEV
                            .build()
                    }

                    // Any other unsigned call from this pallet is invalid.
                    _ => InvalidTransaction::Call.into(),
                }
            }
        }



    // #[pallet::validate_unsigned]
    // impl<T: Config> ValidateUnsigned for Pallet<T> {
    //     type Call = Call<T>;

    //     fn validate_unsigned(
    //         _source: sp_runtime::transaction_validity::TransactionSource,
    //         call: &Self::Call,
    //     ) -> sp_runtime::transaction_validity::TransactionValidity {
    //         use sp_runtime::transaction_validity::{InvalidTransaction, ValidTransaction};

    //         match call {
    //             // This is the only unsigned entry point.
    //             Call::execute_revealed { id, .. } => {
    //                 // Mark this unsigned tx as valid for the pool & block builder.
    //                 // - no source check: works for pool, block building, and block import
    //                 // - propagate(true): gossip so *whoever* authors next block sees it
    //                 // - provides(id): lets the pool deduplicate by this id
    //                 ValidTransaction::with_tag_prefix("mev-shield-exec")
    //                     .priority(u64::MAX) // always prefer executes when present
    //                     .longevity(1)       // only for the very next block
    //                     .and_provides(id)   // crucial: at least one tag
    //                     .propagate(true)    // <-- changed from false to true
    //                     .build()
    //             }

    //             // Any other unsigned call from this pallet is invalid.
    //             _ => InvalidTransaction::Call.into(),
    //         }
    //     }
    // }
}
