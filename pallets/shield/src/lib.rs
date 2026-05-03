// pallets/mev-shield/src/lib.rs
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{vec, vec::Vec};

use chacha20poly1305::{
    KeyInit, XChaCha20Poly1305, XNonce,
    aead::{Aead, Payload},
};
use codec::{Decode, Encode};
use frame_support::{
    dispatch::{GetDispatchInfo, PostDispatchInfo},
    pallet_prelude::*,
    traits::{ConstU64, IsSubType},
};
use frame_system::{ensure_none, ensure_root, ensure_signed, pallet_prelude::*};
use mev_shield_ibe_runtime_api::{
    DkgAuthorityInfo, DkgAuthorityRegistration, DkgConsensusKeyKind, DkgConsensusSource,
    DkgTransportKeyRegistration, EpochDkgPlan, EpochDkgPublication,
};
use ml_kem::{
    Ciphertext, EncodedSizeUser, MlKem768, MlKem768Params,
    kem::{Decapsulate, DecapsulationKey},
};
use sp_io::hashing::twox_128;
use sp_runtime::{
    traits::{
        Applyable, Block as BlockT, Checkable, Dispatchable, Hash, SaturatedConversion, Saturating,
        Zero,
    },
    transaction_validity::{
        InvalidTransaction, TransactionSource, TransactionValidity, ValidTransaction,
    },
};
use stp_mev_shield_ibe::{
    BoundedMasterPublicKey, IbeBlockDecryptionKeyV1, IbeEncryptedExtrinsicV1, IbeEpochPublicKey,
    IbePendingIdentity, KEY_ID_LEN, MEV_SHIELD_IBE_VERSION, block_key_storage_key,
};
use stp_shield::{
    INHERENT_IDENTIFIER, InherentType, LOG_TARGET, MLKEM768_ENC_KEY_LEN, ShieldEncKey,
    ShieldedTransaction,
};
use subtensor_macros::freeze_struct;

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::WeightInfo;

#[cfg(test)]
pub mod mock;

#[cfg(test)]
mod tests;

mod extension;
mod migrations;

pub use extension::CheckShieldedTxValidity;

type MigrationKeyMaxLen = ConstU32<128>;

type ExtrinsicOf<Block> = <Block as BlockT>::Extrinsic;
type CheckedOf<T, Context> = <T as Checkable<Context>>::Checked;
type ApplyableCallOf<T> = <T as Applyable>::Call;

const MAX_EXTRINSIC_DEPTH: u32 = 8;

/// Weight for `store_encrypted`, intentionally set higher than the benchmark
/// to discourage abuse of the encrypted extrinsic queue.
const STORE_ENCRYPTED_WEIGHT: u64 = 20_000_000_000;

pub const IBE_TARGET_LOOKAHEAD_BLOCKS: u64 = 2;
pub const IBE_DKG_EPOCHS_AHEAD: u64 = 2;

pub trait IbeDkgAuthorityProvider {
    fn authorities_for_epoch(epoch: u64) -> Vec<DkgAuthorityInfo>;
    fn consensus_source_for_epoch(epoch: u64) -> DkgConsensusSource;
    fn verify_authority_signature(
        authority_id: &[u8],
        payload_hash: sp_core::H256,
        signature: &[u8],
    ) -> bool;
    fn verify_dkg_authority_registration(registration: &DkgAuthorityRegistration) -> bool;
}

impl IbeDkgAuthorityProvider for () {
    fn authorities_for_epoch(_epoch: u64) -> Vec<DkgAuthorityInfo> {
        Vec::new()
    }
    fn consensus_source_for_epoch(_epoch: u64) -> DkgConsensusSource {
        DkgConsensusSource::PoaAuraRootValidators
    }
    fn verify_authority_signature(
        _authority_id: &[u8],
        _payload_hash: sp_core::H256,
        _signature: &[u8],
    ) -> bool {
        false
    }
    fn verify_dkg_authority_registration(_registration: &DkgAuthorityRegistration) -> bool {
        false
    }
}

pub fn store_encrypted_weight() -> Weight {
    Weight::from_parts(STORE_ENCRYPTED_WEIGHT, 0)
}

/// Trait for decrypting stored extrinsics before dispatch.
pub trait ExtrinsicDecryptor<RuntimeCall> {
    /// Decrypt the stored bytes and return the decoded RuntimeCall.
    fn decrypt(data: &[u8]) -> Result<RuntimeCall, DispatchError>;
}

/// Default implementation that always returns an error.
impl<RuntimeCall> ExtrinsicDecryptor<RuntimeCall> for () {
    fn decrypt(_data: &[u8]) -> Result<RuntimeCall, DispatchError> {
        Err(DispatchError::Other("ExtrinsicDecryptor not implemented"))
    }
}

pub enum IbeDecryptOutcome<InnerExtrinsic> {
    NotReady,
    InvalidAfterKeyAvailable,
    Ready(InnerExtrinsic),
}

pub trait IbeEncryptedTxDecryptor<InnerExtrinsic> {
    fn decrypt(data: &[u8]) -> IbeDecryptOutcome<InnerExtrinsic>;
}

impl<InnerExtrinsic> IbeEncryptedTxDecryptor<InnerExtrinsic> for () {
    fn decrypt(_data: &[u8]) -> IbeDecryptOutcome<InnerExtrinsic> {
        IbeDecryptOutcome::InvalidAfterKeyAvailable
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct IbeAppliedExtrinsic {
    pub consumed_weight: Weight,
    pub success: bool,
}

pub trait DecryptedExtrinsicExecutor<InnerExtrinsic> {
    fn dispatch_info(inner: &InnerExtrinsic) -> Option<frame_support::dispatch::DispatchInfo>;

    fn apply(inner: InnerExtrinsic) -> IbeAppliedExtrinsic;
}

impl<InnerExtrinsic> DecryptedExtrinsicExecutor<InnerExtrinsic> for () {
    fn dispatch_info(_inner: &InnerExtrinsic) -> Option<frame_support::dispatch::DispatchInfo> {
        None
    }

    fn apply(_inner: InnerExtrinsic) -> IbeAppliedExtrinsic {
        IbeAppliedExtrinsic {
            consumed_weight: Weight::zero(),
            success: false,
        }
    }
}

pub trait IbeKeyVerifier<HashT> {
    fn verify_block_identity_key(
        genesis_hash: HashT,
        epoch_key: &IbeEpochPublicKey,
        target_block: u64,
        identity_decryption_key: &[u8],
    ) -> bool;
}

impl<HashT> IbeKeyVerifier<HashT> for () {
    fn verify_block_identity_key(
        _genesis_hash: HashT,
        _epoch_key: &IbeEpochPublicKey,
        _target_block: u64,
        _identity_decryption_key: &[u8],
    ) -> bool {
        false
    }
}

enum PendingProcess {
    Continue(Weight),
    Break(Weight),
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The identifier type for an authority.
        type AuthorityId: Member + Parameter + MaybeSerializeDeserialize + MaxEncodedLen;

        /// A way to find the current and next block author.
        type FindAuthors: FindAuthors<Self>;

        /// The overarching call type for dispatching stored extrinsics.
        type RuntimeCall: Parameter
            + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin, PostInfo = PostDispatchInfo>
            + GetDispatchInfo;

        /// Decryptor for stored extrinsics.
        type ExtrinsicDecryptor: ExtrinsicDecryptor<<Self as Config>::RuntimeCall>;

        /// Signed inner extrinsic decrypted by MEVShield v2 threshold IBE.
        type InnerExtrinsic: Parameter + Encode + Decode;

        /// Decryptor for MEVShield v2 threshold-IBE envelopes.
        type IbeEncryptedTxDecryptor: IbeEncryptedTxDecryptor<Self::InnerExtrinsic>;

        /// Applies decrypted signed inner extrinsics.
        type DecryptedExtrinsicExecutor: DecryptedExtrinsicExecutor<Self::InnerExtrinsic>;

        /// Verifies a reconstructed IBE block identity key against the epoch master public key.
        type IbeKeyVerifier: IbeKeyVerifier<Self::Hash>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        #[pallet::constant]
        type EpochLength: frame_support::traits::Get<u64>;
        #[pallet::constant]
        type MaxDkgAtoms: frame_support::traits::Get<u32>;
        #[pallet::constant]
        type MaxPendingIbePerSender: frame_support::traits::Get<u32>;
        type IbeDkgAuthorityProvider: crate::IbeDkgAuthorityProvider;
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// Current block author's ML-KEM-768 encapsulation key (internal, not for encryption).
    #[pallet::storage]
    pub type CurrentKey<T> = StorageValue<_, ShieldEncKey, OptionQuery>;

    /// Next block author's key, staged here before promoting to `CurrentKey`.
    #[pallet::storage]
    pub type PendingKey<T> = StorageValue<_, ShieldEncKey, OptionQuery>;

    /// Key users should encrypt with (N+2 author's key).
    #[pallet::storage]
    pub type NextKey<T> = StorageValue<_, ShieldEncKey, OptionQuery>;

    /// Per-author ML-KEM-768 encapsulation key, updated each time the author produces a block.
    #[pallet::storage]
    pub type AuthorKeys<T: Config> =
        StorageMap<_, Twox64Concat, T::AuthorityId, ShieldEncKey, OptionQuery>;

    /// Block number at which `PendingKey` is no longer valid (exclusive upper bound).
    /// Updated every block during rotation.
    #[pallet::storage]
    pub type PendingKeyExpiresAt<T: Config> = StorageValue<_, BlockNumberFor<T>, OptionQuery>;

    /// Block number at which `NextKey` is no longer valid (exclusive upper bound).
    /// Updated every block during rotation.
    #[pallet::storage]
    pub type NextKeyExpiresAt<T: Config> = StorageValue<_, BlockNumberFor<T>, OptionQuery>;

    /// Stores whether some migration has been run.
    #[pallet::storage]
    pub type HasMigrationRun<T: Config> =
        StorageMap<_, Identity, BoundedVec<u8, MigrationKeyMaxLen>, bool, ValueQuery>;

    /// Maximum size of a single encoded call.
    pub type MaxEncryptedCallSize = ConstU32<8192>;

    /// Default maximum number of pending extrinsics.
    pub type DefaultMaxPendingExtrinsics = ConstU32<100>;

    /// Configurable maximum number of pending extrinsics.
    /// Defaults to 100 if not explicitly set via `set_max_pending_extrinsics`.
    #[pallet::storage]
    pub type MaxPendingExtrinsicsLimit<T: Config> =
        StorageValue<_, u32, ValueQuery, DefaultMaxPendingExtrinsics>;

    /// Default extrinsic lifetime in blocks.
    pub const DEFAULT_EXTRINSIC_LIFETIME: u32 = 10;

    /// Configurable extrinsic lifetime (max block difference between submission and execution).
    /// Defaults to 10 blocks if not explicitly set.
    #[pallet::storage]
    pub type ExtrinsicLifetime<T: Config> =
        StorageValue<_, u32, ValueQuery, ConstU32<DEFAULT_EXTRINSIC_LIFETIME>>;

    /// Default maximum weight allowed for on_initialize processing.
    pub const DEFAULT_ON_INITIALIZE_WEIGHT: u64 = 500_000_000_000;

    /// Absolute maximum weight for on_initialize: half the total block weight (2s of 4s).
    pub const MAX_ON_INITIALIZE_WEIGHT: u64 = 2_000_000_000_000;

    /// Configurable maximum weight for on_initialize processing.
    /// Defaults to 500_000_000_000 ref_time if not explicitly set.
    #[pallet::storage]
    pub type OnInitializeWeight<T: Config> =
        StorageValue<_, u64, ValueQuery, ConstU64<DEFAULT_ON_INITIALIZE_WEIGHT>>;

    /// Default maximum weight for a single extrinsic.
    pub const DEFAULT_MAX_EXTRINSIC_WEIGHT: u64 = 50_000_000_000;

    /// Configurable maximum weight for a single extrinsic dispatched during on_initialize.
    /// Extrinsics exceeding this limit are removed from the queue.
    #[pallet::storage]
    pub type MaxExtrinsicWeight<T: Config> =
        StorageValue<_, u64, ValueQuery, ConstU64<DEFAULT_MAX_EXTRINSIC_WEIGHT>>;

    /// A pending extrinsic stored for later execution.
    #[freeze_struct("f13d2a9d7bd4767d")]
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Debug)]
    #[scale_info(skip_type_params(T))]
    pub struct PendingExtrinsic<T: Config> {
        /// The account that submitted the extrinsic.
        pub who: T::AccountId,

        /// The encoded encrypted envelope/call data.
        pub encrypted_call: BoundedVec<u8, MaxEncryptedCallSize>,

        /// The block number when the extrinsic was submitted.
        pub submitted_at: BlockNumberFor<T>,
    }

    /// Storage map for encrypted extrinsics to be executed in on_initialize.
    /// Uses u32 index for O(1) insertion and removal. Count is maintained automatically.
    #[pallet::storage]
    pub type PendingExtrinsics<T: Config> =
        CountedStorageMap<_, Identity, u32, PendingExtrinsic<T>, OptionQuery>;

    /// Next index to use when inserting a pending extrinsic (unique auto-increment).
    #[pallet::storage]
    pub type NextPendingExtrinsicIndex<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// MEVShield v2 threshold-IBE epoch master public keys.

    #[freeze_struct("5a9a72dffce049d7")]
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Debug)]
    #[scale_info(skip_type_params(T))]
    pub struct PendingIbeMeta<T: Config> {
        pub epoch: u64,
        pub target_block: u64,
        pub key_id: [u8; KEY_ID_LEN],
        pub commitment: sp_core::H256,
        pub submitted_at: BlockNumberFor<T>,
        pub submitted_tx_index: u32,
        pub submitter: T::AccountId,
    }

    #[pallet::storage]
    pub type PendingIbeMetadata<T: Config> =
        StorageMap<_, Identity, u32, PendingIbeMeta<T>, OptionQuery>;
    #[pallet::storage]
    pub type PendingIbeCommitments<T: Config> =
        StorageMap<_, Blake2_128Concat, sp_core::H256, u32, OptionQuery>;
    #[pallet::storage]
    pub type PendingIbeBySubmitter<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;
    #[pallet::storage]
    pub type PublishedDkgOutputHashes<T: Config> =
        StorageMap<_, Twox64Concat, u64, sp_core::H256, OptionQuery>;
    #[pallet::storage]
    pub type IbeDkgTransportPublicKeys<T: Config> =
        StorageMap<_, Blake2_128Concat, Vec<u8>, [u8; 32], OptionQuery>;
    #[pallet::storage]
    pub type IbeDkgAuthorityRegistrations<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        Vec<u8>,
        Twox64Concat,
        DkgConsensusKeyKind,
        DkgAuthorityRegistration,
        OptionQuery,
    >;
    #[pallet::storage]
    pub type IbeDkgAuthoritySnapshots<T: Config> =
        StorageMap<_, Twox64Concat, u64, Vec<DkgAuthorityInfo>, ValueQuery>;

    #[pallet::storage]
    pub type IbeEpochKeys<T: Config> =
        StorageMap<_, Twox64Concat, u64, IbeEpochPublicKey, OptionQuery>;

    /// Published/reconstructed MEVShield v2 block identity decryption keys.
    #[pallet::storage]
    pub type IbeBlockDecryptionKeys<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        (u64, u64, [u8; KEY_ID_LEN]),
        IbeBlockDecryptionKeyV1,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Encrypted wrapper accepted.
        EncryptedSubmitted {
            id: T::Hash,
            who: T::AccountId,
        },
        /// Encrypted extrinsic was stored for later execution.
        ExtrinsicStored {
            index: u32,
            who: T::AccountId,
        },
        /// Extrinsic decode failed during on_initialize.
        ExtrinsicDecodeFailed {
            index: u32,
        },
        /// Extrinsic dispatch failed during on_initialize.
        ExtrinsicDispatchFailed {
            index: u32,
            error: DispatchError,
        },
        /// Extrinsic was successfully dispatched during on_initialize.
        ExtrinsicDispatched {
            index: u32,
        },
        /// Extrinsic expired (exceeded max block lifetime).
        ExtrinsicExpired {
            index: u32,
        },
        /// Extrinsic postponed due to missing key or weight limit.
        ExtrinsicPostponed {
            index: u32,
        },
        /// Maximum pending extrinsics limit was updated.
        MaxPendingExtrinsicsNumberSet {
            value: u32,
        },
        /// Maximum on_initialize weight was updated.
        OnInitializeWeightSet {
            value: u64,
        },
        /// Extrinsic lifetime was updated.
        ExtrinsicLifetimeSet {
            value: u32,
        },
        /// Maximum per-extrinsic weight was updated.
        MaxExtrinsicWeightSet {
            value: u64,
        },
        /// Extrinsic exceeded the per-extrinsic weight limit and was removed.
        ExtrinsicWeightExceeded {
            index: u32,
        },
        /// MEVShield v2 IBE encrypted extrinsic accepted into the pending queue.
        IbeEncryptedSubmitted {
            index: u32,
            who: T::AccountId,
            epoch: u64,
            target_block: u64,
            key_id: [u8; KEY_ID_LEN],
            commitment: sp_core::H256,
        },
        /// MEVShield v2 IBE epoch master public key was set.
        IbeEpochPublicKeySet {
            epoch: u64,
            key_id: [u8; KEY_ID_LEN],
        },
        /// MEVShield v2 block identity decryption key was submitted.
        IbeBlockDecryptionKeySubmitted {
            epoch: u64,
            target_block: u64,
            key_id: [u8; KEY_ID_LEN],
        },
        IbeEpochDkgPublicKeyPublished {
            epoch: u64,
            key_id: [u8; KEY_ID_LEN],
            attested_weight: u128,
        },
        IbeDkgTransportKeyRegistered {
            authority_id: Vec<u8>,
            dkg_x25519_public_key: [u8; 32],
        },
        IbeDkgAuthorityRegistered {
            hotkey_account_id: Vec<u8>,
            consensus_key_kind: DkgConsensusKeyKind,
            authority_id: Vec<u8>,
            dkg_x25519_public_key: [u8; 32],
        },
        IbeDkgAuthoritySnapshotStored {
            epoch: u64,
            authority_count: u32,
        },

        /// MEVShield v2 encrypted extrinsic was invalid after the block key became available.
        IbeEncryptedExtrinsicInvalid {
            index: u32,
        },
        /// MEVShield v2 encrypted extrinsic consumed its canonical queue position.
        IbeEncryptedExtrinsicExecuted {
            index: u32,
            success: bool,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The announced ML-KEM encapsulation key length is invalid.
        BadEncKeyLen,
        /// Unreachable.
        Unreachable,
        /// Too many pending extrinsics in storage.
        TooManyPendingExtrinsics,
        /// Weight exceeds the absolute maximum (half of total block weight).
        WeightExceedsAbsoluteMax,
        /// Invalid MEVShield v2 IBE envelope.
        BadIbeEnvelope,
        /// Unknown MEVShield v2 IBE epoch.
        UnknownIbeEpoch,
        /// IBE key id does not match the epoch key.
        WrongIbeEpochKey,
        /// The encrypted target block is stale.
        StaleEncryptedTarget,
        /// The IBE block identity key has already been published.
        IbeKeyAlreadyPublished,
        /// IBE block identity key was submitted before its target block.
        IbeKeyTooEarly,
        /// Invalid IBE block identity decryption key.
        InvalidIbeBlockDecryptionKey,
        IbeEpochKeyInactive,
        InvalidIbeTargetWindow,
        DuplicateIbeCommitment,
        TooManyPendingIbeForSender,
        InvalidIbeFinalityPoint,
        BadIbeDkgPublication,
        InsufficientIbeDkgAttestationWeight,
        IbeDkgPublicationAlreadyKnown,
        BadIbeDkgTransportKeyRegistration,
        BadIbeDkgAuthorityRegistration,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_block_number: BlockNumberFor<T>) -> Weight {
            let mut weight = Self::ensure_epoch_ahead_dkg_snapshots();
            weight = weight.saturating_add(Self::process_pending_extrinsics());
            weight
        }

        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            let mut weight = frame_support::weights::Weight::from_parts(0, 0);

            weight = weight.saturating_add(
                migrations::migrate_clear_v1_storage::migrate_clear_v1_storage::<T>(),
            );

            weight
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Rotate the key chain and announce the current author's ML-KEM encapsulation key.
        ///
        /// Called as an inherent every block. `enc_key` is `None` on node failure,
        /// which removes the author from future shielded tx eligibility.
        ///
        /// Key rotation order (using pre-update AuthorKeys):
        /// 1. CurrentKey ← PendingKey
        /// 2. PendingKey ← NextKey
        /// 3. NextKey ← next-next author's key (user-facing)
        /// 4. AuthorKeys[current] ← announced key
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::announce_next_key())]
        pub fn announce_next_key(
            origin: OriginFor<T>,
            enc_key: Option<ShieldEncKey>,
        ) -> DispatchResult {
            ensure_none(origin)?;

            let author = T::FindAuthors::find_current_author().ok_or(Error::<T>::Unreachable)?;
            let now = <frame_system::Pallet<T>>::block_number();

            // 1. CurrentKey ← PendingKey
            if let Some(pending_key) = PendingKey::<T>::take() {
                CurrentKey::<T>::put(pending_key);
            } else {
                CurrentKey::<T>::kill();
            }

            // 2. PendingKey ← NextKey (what was N+2 last block is now N+1)
            if let Some(next_key) = NextKey::<T>::take() {
                PendingKey::<T>::put(next_key);
            } else {
                PendingKey::<T>::kill();
            }

            // 3. NextKey ← next-next author's key
            if let Some(next_next_author) = T::FindAuthors::find_next_next_author()
                && let Some(key) = AuthorKeys::<T>::get(&next_next_author)
            {
                NextKey::<T>::put(key);
            } else {
                NextKey::<T>::kill();
            }

            // 4. Update AuthorKeys after rotations for consistent reads above.
            if let Some(enc_key) = &enc_key {
                ensure!(
                    enc_key.len() == MLKEM768_ENC_KEY_LEN,
                    Error::<T>::BadEncKeyLen
                );
                AuthorKeys::<T>::insert(&author, enc_key.clone());
            } else {
                AuthorKeys::<T>::remove(&author);
            }

            // 5. Set expiration blocks for user-facing keys.
            if PendingKey::<T>::get().is_some() {
                PendingKeyExpiresAt::<T>::put(now + 2u32.into());
            } else {
                PendingKeyExpiresAt::<T>::kill();
            }
            if NextKey::<T>::get().is_some() {
                NextKeyExpiresAt::<T>::put(now + 3u32.into());
            } else {
                NextKeyExpiresAt::<T>::kill();
            }

            Ok(())
        }

        /// Users submit an encrypted wrapper.
        ///
        /// v1:
        /// ciphertext = key_hash || kem_len || kem_ct || nonce || aead_ct
        ///
        /// v2:
        /// ciphertext is SCALE(IbeEncryptedExtrinsicV1), prefixed with the v2 magic.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::submit_encrypted())]
        pub fn submit_encrypted(
            origin: OriginFor<T>,
            ciphertext: BoundedVec<u8, MaxEncryptedCallSize>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            if IbeEncryptedExtrinsicV1::is_v2_prefixed(ciphertext.as_slice()) {
                return Self::submit_encrypted_v2_inner(who, ciphertext);
            }

            let id: T::Hash = T::Hashing::hash_of(&(who.clone(), &ciphertext));
            Self::deposit_event(Event::EncryptedSubmitted { id, who });
            Ok(())
        }

        /// Store an encrypted extrinsic for later execution in on_initialize.
        #[pallet::call_index(2)]
        #[pallet::weight(store_encrypted_weight())]
        pub fn store_encrypted(
            origin: OriginFor<T>,
            encrypted_call: BoundedVec<u8, MaxEncryptedCallSize>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::store_encrypted_inner(who, encrypted_call)
        }

        /// Set the maximum number of pending extrinsics allowed in the queue.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::set_max_pending_extrinsics_number())]
        pub fn set_max_pending_extrinsics_number(
            origin: OriginFor<T>,
            value: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;

            MaxPendingExtrinsicsLimit::<T>::put(value);

            Self::deposit_event(Event::MaxPendingExtrinsicsNumberSet { value });
            Ok(())
        }

        /// Set the maximum weight allowed for on_initialize processing.
        /// Rejects values exceeding the absolute limit (half of total block weight).
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::set_on_initialize_weight())]
        pub fn set_on_initialize_weight(origin: OriginFor<T>, value: u64) -> DispatchResult {
            ensure_root(origin)?;

            ensure!(
                value <= MAX_ON_INITIALIZE_WEIGHT,
                Error::<T>::WeightExceedsAbsoluteMax
            );

            OnInitializeWeight::<T>::put(value);

            Self::deposit_event(Event::OnInitializeWeightSet { value });
            Ok(())
        }

        /// Set the extrinsic lifetime (max blocks between submission and execution).
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::set_stored_extrinsic_lifetime())]
        pub fn set_stored_extrinsic_lifetime(origin: OriginFor<T>, value: u32) -> DispatchResult {
            ensure_root(origin)?;

            ExtrinsicLifetime::<T>::put(value);

            Self::deposit_event(Event::ExtrinsicLifetimeSet { value });
            Ok(())
        }

        /// Set the maximum weight allowed for a single extrinsic during on_initialize processing.
        /// Extrinsics exceeding this limit are removed from the queue.
        /// Rejects values exceeding the absolute limit.
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::set_max_extrinsic_weight())]
        pub fn set_max_extrinsic_weight(origin: OriginFor<T>, value: u64) -> DispatchResult {
            ensure_root(origin)?;

            ensure!(
                value <= MAX_ON_INITIALIZE_WEIGHT,
                Error::<T>::WeightExceedsAbsoluteMax
            );

            MaxExtrinsicWeight::<T>::put(value);

            Self::deposit_event(Event::MaxExtrinsicWeightSet { value });
            Ok(())
        }

        /// Set a MEVShield v2 IBE epoch public key.
        ///
        /// In production this should be called by the DKG output pipeline or governance/root
        /// after epoch-ahead DKG completes.
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::set_max_pending_extrinsics_number())]
        pub fn set_ibe_epoch_public_key(
            origin: OriginFor<T>,
            epoch_key: IbeEpochPublicKey,
        ) -> DispatchResult {
            ensure_root(origin)?;

            IbeEpochKeys::<T>::insert(epoch_key.epoch, epoch_key.clone());

            Self::deposit_event(Event::IbeEpochPublicKeySet {
                epoch: epoch_key.epoch,
                key_id: epoch_key.key_id,
            });

            Ok(())
        }

        /// Submit a reconstructed MEVShield v2 block identity decryption key.
        ///
        /// This accepts either:
        /// - signed origin, for manual/operator submission;
        /// - unsigned origin, for local node submission produced by the v2 share pool.
        ///
        /// Unsigned transaction-pool validity is provided by `ValidateUnsigned` below.
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::submit_encrypted())]
        pub fn submit_block_decryption_key(
            origin: OriginFor<T>,
            key: IbeBlockDecryptionKeyV1,
        ) -> DispatchResult {
            if ensure_signed(origin.clone()).is_err() {
                ensure_none(origin)?;
            }
            Self::validate_ibe_block_decryption_key_for_submission(&key)?;
            IbeBlockDecryptionKeys::<T>::insert(
                block_key_storage_key(key.epoch, key.target_block, key.key_id),
                key.clone(),
            );
            Self::deposit_event(Event::IbeBlockDecryptionKeySubmitted {
                epoch: key.epoch,
                target_block: key.target_block,
                key_id: key.key_id,
            });
            Ok(())
        }

        #[pallet::call_index(9)]
        #[pallet::weight(T::WeightInfo::set_max_pending_extrinsics_number())]
        pub fn publish_ibe_epoch_public_key(
            origin: OriginFor<T>,
            publication: EpochDkgPublication,
        ) -> DispatchResult {
            ensure_none(origin)?;
            Self::publish_ibe_epoch_public_key_inner(publication)
        }

        #[pallet::call_index(10)]
        #[pallet::weight(T::WeightInfo::set_max_pending_extrinsics_number())]
        pub fn submit_ibe_dkg_transport_key(
            origin: OriginFor<T>,
            registration: DkgTransportKeyRegistration,
        ) -> DispatchResult {
            ensure_none(origin)?;
            Self::submit_ibe_dkg_transport_key_inner(registration)
        }

        #[pallet::call_index(11)]
        #[pallet::weight(T::WeightInfo::set_max_pending_extrinsics_number())]
        pub fn submit_ibe_dkg_authority_registration(
            origin: OriginFor<T>,
            registration: DkgAuthorityRegistration,
        ) -> DispatchResult {
            ensure_none(origin)?;
            Self::submit_ibe_dkg_authority_registration_inner(registration)
        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> sp_runtime::traits::ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            if <Self as frame_support::pallet_prelude::ProvideInherent>::is_inherent(call) {
                return match source {
                    TransactionSource::InBlock => Ok(ValidTransaction::default()),
                    _ => InvalidTransaction::Call.into(),
                };
            }

            if let Call::publish_ibe_epoch_public_key { publication } = call {
                if Self::verify_epoch_dkg_publication(publication).is_err() {
                    return InvalidTransaction::BadProof.into();
                }
                return ValidTransaction::with_tag_prefix("MevShieldIbeEpochDkg")
                    .priority(2_000_000)
                    .and_provides((
                        b"mev-shield-ibe-epoch-dkg".as_slice(),
                        publication.epoch,
                        publication.key_id,
                    ))
                    .longevity(256)
                    .propagate(true)
                    .build();
            }

            let Call::submit_block_decryption_key { key } = call else {
                return InvalidTransaction::Call.into();
            };

            if Self::validate_ibe_block_decryption_key_for_submission(key).is_err() {
                return InvalidTransaction::BadProof.into();
            }

            ValidTransaction::with_tag_prefix("MevShieldIbeBlockKey")
                .priority(1_000_000)
                .and_provides((
                    b"mev-shield-ibe-block-key".as_slice(),
                    key.epoch,
                    key.target_block,
                    key.key_id,
                ))
                .longevity(64)
                .propagate(true)
                .build()
        }
    }

    #[pallet::inherent]
    impl<T: Config> ProvideInherent for Pallet<T> {
        type Call = Call<T>;
        type Error = sp_inherents::MakeFatalError<()>;

        const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

        fn create_inherent(data: &InherentData) -> Option<Self::Call> {
            let enc_key = data
                .get_data::<InherentType>(&INHERENT_IDENTIFIER)
                .inspect_err(
                    |e| log::debug!(target: LOG_TARGET, "Failed to get shielded enc key inherent data: {:?}", e),
                )
                .ok()??;

            Some(Call::announce_next_key { enc_key })
        }

        fn is_inherent(call: &Self::Call) -> bool {
            matches!(call, Call::announce_next_key { .. })
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn current_ibe_epoch() -> u64 {
        let n: u64 = frame_system::Pallet::<T>::block_number().saturated_into::<u64>();
        let epoch_len = T::EpochLength::get().max(1);
        n / epoch_len
    }
    pub fn epoch_bounds(epoch: u64) -> (u64, u64) {
        let len = T::EpochLength::get().max(1);
        let first = epoch.saturating_mul(len);
        let last = first.saturating_add(len.saturating_sub(1));
        (first, last)
    }
    pub fn ensure_epoch_ahead_dkg_snapshots() -> frame_support::weights::Weight {
        let current = Self::current_ibe_epoch();
        let mut reads = 0u64;
        let mut writes = 0u64;
        for offset in 0..=IBE_DKG_EPOCHS_AHEAD {
            let epoch = current.saturating_add(offset);
            reads = reads.saturating_add(1);
            if !IbeDkgAuthoritySnapshots::<T>::get(epoch).is_empty() {
                continue;
            }
            let authorities = T::IbeDkgAuthorityProvider::authorities_for_epoch(epoch);
            if authorities.is_empty() {
                continue;
            }
            let authority_count = authorities.len() as u32;
            IbeDkgAuthoritySnapshots::<T>::insert(epoch, authorities);
            writes = writes.saturating_add(1);
            Self::deposit_event(Event::IbeDkgAuthoritySnapshotStored {
                epoch,
                authority_count,
            });
        }
        <T as frame_system::Config>::DbWeight::get().reads_writes(reads, writes)
    }
    pub fn dkg_authorities_for_plan(epoch: u64) -> Vec<DkgAuthorityInfo> {
        let snapshot = IbeDkgAuthoritySnapshots::<T>::get(epoch);
        if !snapshot.is_empty() {
            snapshot
        } else {
            T::IbeDkgAuthorityProvider::authorities_for_epoch(epoch)
        }
    }
    pub fn dkg_transport_key_payload_hash(
        authority_id: &[u8],
        dkg_x25519_public_key: &[u8; 32],
    ) -> sp_core::H256 {
        sp_core::H256::from(sp_core::hashing::blake2_256(
            &(
                b"bittensor.mev-shield.v2.dkg.transport-key",
                authority_id,
                dkg_x25519_public_key,
            )
                .encode(),
        ))
    }
    pub fn verify_dkg_transport_key_registration(
        registration: &DkgTransportKeyRegistration,
    ) -> DispatchResult {
        let payload = Self::dkg_transport_key_payload_hash(
            &registration.authority_id,
            &registration.dkg_x25519_public_key,
        );
        ensure!(
            T::IbeDkgAuthorityProvider::verify_authority_signature(
                &registration.authority_id,
                payload,
                &registration.signature,
            ),
            Error::<T>::BadIbeDkgTransportKeyRegistration
        );
        Ok(())
    }
    pub fn submit_ibe_dkg_transport_key_inner(
        registration: DkgTransportKeyRegistration,
    ) -> DispatchResult {
        Self::verify_dkg_transport_key_registration(&registration)?;
        IbeDkgTransportPublicKeys::<T>::insert(
            registration.authority_id.clone(),
            registration.dkg_x25519_public_key,
        );
        Self::deposit_event(Event::IbeDkgTransportKeyRegistered {
            authority_id: registration.authority_id,
            dkg_x25519_public_key: registration.dkg_x25519_public_key,
        });
        Ok(())
    }
    pub fn verify_dkg_authority_registration(
        registration: &DkgAuthorityRegistration,
    ) -> DispatchResult {
        ensure!(
            T::IbeDkgAuthorityProvider::verify_dkg_authority_registration(registration),
            Error::<T>::BadIbeDkgAuthorityRegistration
        );
        Ok(())
    }
    pub fn submit_ibe_dkg_authority_registration_inner(
        registration: DkgAuthorityRegistration,
    ) -> DispatchResult {
        Self::verify_dkg_authority_registration(&registration)?;
        IbeDkgAuthorityRegistrations::<T>::insert(
            registration.hotkey_account_id.clone(),
            registration.consensus_key_kind,
            registration.clone(),
        );
        IbeDkgTransportPublicKeys::<T>::insert(
            registration.consensus_authority_id.clone(),
            registration.dkg_x25519_public_key,
        );
        Self::deposit_event(Event::IbeDkgAuthorityRegistered {
            hotkey_account_id: registration.hotkey_account_id,
            consensus_key_kind: registration.consensus_key_kind,
            authority_id: registration.consensus_authority_id,
            dkg_x25519_public_key: registration.dkg_x25519_public_key,
        });
        Ok(())
    }
    pub fn next_epoch_dkg_plan() -> Option<EpochDkgPlan> {
        let epoch = Self::current_ibe_epoch().saturating_add(IBE_DKG_EPOCHS_AHEAD);
        let authorities = Self::dkg_authorities_for_plan(epoch);
        if authorities.is_empty() {
            return None;
        }
        let (first_block, last_block) = Self::epoch_bounds(epoch);
        Some(EpochDkgPlan {
            epoch,
            first_block,
            last_block,
            consensus_source: T::IbeDkgAuthorityProvider::consensus_source_for_epoch(epoch),
            max_atoms: T::MaxDkgAtoms::get(),
            authorities,
        })
    }
    pub fn active_epoch_dkg_plan() -> Option<EpochDkgPlan> {
        let epoch = Self::current_ibe_epoch();
        let authorities = Self::dkg_authorities_for_plan(epoch);
        if authorities.is_empty() {
            return None;
        }
        let (first_block, last_block) = Self::epoch_bounds(epoch);
        Some(EpochDkgPlan {
            epoch,
            first_block,
            last_block,
            consensus_source: T::IbeDkgAuthorityProvider::consensus_source_for_epoch(epoch),
            max_atoms: T::MaxDkgAtoms::get(),
            authorities,
        })
    }
    pub fn dkg_public_output_hash(publication: &EpochDkgPublication) -> sp_core::H256 {
        sp_core::H256::from(sp_core::hashing::blake2_256(
            &(
                b"bittensor.mev-shield.v2.dkg.public-output",
                publication.epoch,
                publication.key_id,
                publication.first_block,
                publication.last_block,
                publication.consensus_source,
                &publication.master_public_key,
                publication.total_weight,
                publication.threshold_weight,
            )
                .encode(),
        ))
    }
    pub fn dkg_attestation_payload_hash(
        epoch: u64,
        key_id: [u8; KEY_ID_LEN],
        public_output_hash: sp_core::H256,
        authority_id: &[u8],
        stake: u128,
    ) -> sp_core::H256 {
        sp_core::H256::from(sp_core::hashing::blake2_256(
            &(
                b"bittensor.mev-shield.v2.dkg.output-attestation",
                epoch,
                key_id,
                public_output_hash,
                authority_id,
                stake,
            )
                .encode(),
        ))
    }
    pub fn verify_epoch_dkg_publication(publication: &EpochDkgPublication) -> DispatchResult {
        let expected_hash = Self::dkg_public_output_hash(publication);
        ensure!(
            publication.public_output_hash == expected_hash,
            Error::<T>::BadIbeDkgPublication
        );

        let plan = if let Some(next) = Self::next_epoch_dkg_plan() {
            if next.epoch == publication.epoch {
                next
            } else {
                Self::active_epoch_dkg_plan().ok_or(Error::<T>::BadIbeDkgPublication)?
            }
        } else {
            Self::active_epoch_dkg_plan().ok_or(Error::<T>::BadIbeDkgPublication)?
        };

        ensure!(
            publication.epoch == plan.epoch,
            Error::<T>::BadIbeDkgPublication
        );
        ensure!(
            publication.consensus_source == plan.consensus_source,
            Error::<T>::BadIbeDkgPublication
        );
        ensure!(
            publication.first_block == plan.first_block
                && publication.last_block == plan.last_block,
            Error::<T>::BadIbeDkgPublication
        );
        ensure!(
            publication.total_weight >= publication.threshold_weight
                && publication.threshold_weight > 0,
            Error::<T>::BadIbeDkgPublication
        );
        ensure!(
            !IbeEpochKeys::<T>::contains_key(publication.epoch)
                && !PublishedDkgOutputHashes::<T>::contains_key(publication.epoch),
            Error::<T>::IbeDkgPublicationAlreadyKnown
        );

        let mut by_authority = sp_std::collections::btree_map::BTreeMap::<Vec<u8>, u128>::new();
        for a in &plan.authorities {
            by_authority.insert(a.authority_id.clone(), a.stake);
        }

        let total_stake = plan
            .authorities
            .iter()
            .fold(0u128, |acc, a| acc.saturating_add(a.stake));
        let threshold_stake = total_stake.saturating_mul(2) / 3 + 1;

        let mut attested_stake = 0u128;
        let mut seen = sp_std::collections::btree_set::BTreeSet::<Vec<u8>>::new();
        for att in &publication.attestations {
            if !seen.insert(att.authority_id.clone()) {
                continue;
            }
            let Some(expected_stake) = by_authority.get(&att.authority_id).copied() else {
                continue;
            };
            if expected_stake != att.stake || att.public_output_hash != expected_hash {
                continue;
            }
            let payload = Self::dkg_attestation_payload_hash(
                publication.epoch,
                publication.key_id,
                expected_hash,
                &att.authority_id,
                att.stake,
            );
            if T::IbeDkgAuthorityProvider::verify_authority_signature(
                &att.authority_id,
                payload,
                &att.signature,
            ) {
                attested_stake = attested_stake.saturating_add(att.stake);
            }
        }

        ensure!(
            attested_stake >= threshold_stake,
            Error::<T>::InsufficientIbeDkgAttestationWeight
        );
        Ok(())
    }
    pub fn publish_ibe_epoch_public_key_inner(publication: EpochDkgPublication) -> DispatchResult {
        Self::verify_epoch_dkg_publication(&publication)?;
        let master_public_key: BoundedMasterPublicKey = publication
            .master_public_key
            .clone()
            .try_into()
            .map_err(|_| Error::<T>::BadIbeDkgPublication)?;

        let epoch_key = IbeEpochPublicKey {
            epoch: publication.epoch,
            key_id: publication.key_id,
            master_public_key,
            total_weight: publication.total_weight,
            threshold_weight: publication.threshold_weight,
            first_block: publication.first_block,
            last_block: publication.last_block,
        };

        IbeEpochKeys::<T>::insert(publication.epoch, epoch_key);
        PublishedDkgOutputHashes::<T>::insert(
            publication.epoch,
            Self::dkg_public_output_hash(&publication),
        );
        let attested_weight = publication
            .attestations
            .iter()
            .fold(0u128, |acc, a| acc.saturating_add(a.stake));
        Self::deposit_event(Event::IbeEpochDkgPublicKeyPublished {
            epoch: publication.epoch,
            key_id: publication.key_id,
            attested_weight,
        });
        Ok(())
    }
    pub fn after_v2_pending_push(
        who: &T::AccountId,
        index: u32,
        envelope: &IbeEncryptedExtrinsicV1,
    ) -> DispatchResult {
        ensure!(
            PendingIbeBySubmitter::<T>::get(who) < T::MaxPendingIbePerSender::get(),
            Error::<T>::TooManyPendingIbeForSender
        );
        let submitted_at = frame_system::Pallet::<T>::block_number();
        let submitted_tx_index = frame_system::Pallet::<T>::extrinsic_index().unwrap_or_default();
        PendingIbeMetadata::<T>::insert(
            index,
            PendingIbeMeta::<T> {
                epoch: envelope.epoch,
                target_block: envelope.target_block,
                key_id: envelope.key_id,
                commitment: envelope.commitment,
                submitted_at,
                submitted_tx_index,
                submitter: who.clone(),
            },
        );
        PendingIbeCommitments::<T>::insert(envelope.commitment, index);
        PendingIbeBySubmitter::<T>::mutate(who, |n| *n = n.saturating_add(1));
        Ok(())
    }
    pub fn remove_pending_index(index: u32) {
        PendingExtrinsics::<T>::remove(index);
        if let Some(meta) = PendingIbeMetadata::<T>::take(index) {
            PendingIbeCommitments::<T>::remove(meta.commitment);
            PendingIbeBySubmitter::<T>::mutate(meta.submitter, |n| *n = n.saturating_sub(1));
        }
    }

    fn submit_encrypted_v2_inner(
        who: T::AccountId,
        encrypted_call: BoundedVec<u8, MaxEncryptedCallSize>,
    ) -> DispatchResult {
        let envelope = IbeEncryptedExtrinsicV1::decode_v2(encrypted_call.as_slice())
            .map_err(|_| Error::<T>::BadIbeEnvelope)?;
        Self::validate_v2_envelope_for_submission(&envelope)?;
        ensure!(
            PendingIbeBySubmitter::<T>::get(&who) < T::MaxPendingIbePerSender::get(),
            Error::<T>::TooManyPendingIbeForSender
        );
        let index = Self::store_pending_encrypted(who.clone(), encrypted_call)?;
        Self::after_v2_pending_push(&who, index, &envelope)?;
        Self::deposit_event(Event::IbeEncryptedSubmitted {
            index,
            who,
            epoch: envelope.epoch,
            target_block: envelope.target_block,
            key_id: envelope.key_id,
            commitment: envelope.commitment,
        });
        Ok(())
    }

    fn store_encrypted_inner(
        who: T::AccountId,
        encrypted_call: BoundedVec<u8, MaxEncryptedCallSize>,
    ) -> DispatchResult {
        if IbeEncryptedExtrinsicV1::is_v2_prefixed(encrypted_call.as_slice()) {
            let envelope = IbeEncryptedExtrinsicV1::decode_v2(encrypted_call.as_slice())
                .map_err(|_| Error::<T>::BadIbeEnvelope)?;
            Self::validate_v2_envelope_for_submission(&envelope)?;
            ensure!(
                PendingIbeBySubmitter::<T>::get(&who) < T::MaxPendingIbePerSender::get(),
                Error::<T>::TooManyPendingIbeForSender
            );
            let index = Self::store_pending_encrypted(who.clone(), encrypted_call)?;
            Self::after_v2_pending_push(&who, index, &envelope)?;
            Self::deposit_event(Event::ExtrinsicStored { index, who });
            return Ok(());
        }
        let index = Self::store_pending_encrypted(who.clone(), encrypted_call)?;
        Self::deposit_event(Event::ExtrinsicStored { index, who });
        Ok(())
    }

    pub fn validate_v2_envelope_for_submission(
        envelope: &IbeEncryptedExtrinsicV1,
    ) -> DispatchResult {
        ensure!(
            envelope.version == MEV_SHIELD_IBE_VERSION,
            Error::<T>::BadIbeEnvelope
        );
        let current_block_u64: u64 =
            frame_system::Pallet::<T>::block_number().saturated_into::<u64>();
        ensure!(
            envelope.target_block == current_block_u64.saturating_add(IBE_TARGET_LOOKAHEAD_BLOCKS),
            Error::<T>::InvalidIbeTargetWindow
        );
        ensure!(
            !PendingIbeCommitments::<T>::contains_key(envelope.commitment),
            Error::<T>::DuplicateIbeCommitment
        );
        let epoch_key =
            IbeEpochKeys::<T>::get(envelope.epoch).ok_or(Error::<T>::UnknownIbeEpoch)?;
        ensure!(
            epoch_key.key_id == envelope.key_id,
            Error::<T>::WrongIbeEpochKey
        );
        ensure!(
            envelope.target_block >= epoch_key.first_block
                && envelope.target_block <= epoch_key.last_block,
            Error::<T>::IbeEpochKeyInactive
        );
        ensure!(
            IbeBlockDecryptionKeys::<T>::get(block_key_storage_key(
                envelope.epoch,
                envelope.target_block,
                envelope.key_id,
            ))
            .is_none(),
            Error::<T>::IbeKeyAlreadyPublished
        );
        Ok(())
    }

    pub fn validate_ibe_block_decryption_key_for_submission(
        key: &IbeBlockDecryptionKeyV1,
    ) -> DispatchResult {
        ensure!(
            key.version == MEV_SHIELD_IBE_VERSION,
            Error::<T>::InvalidIbeBlockDecryptionKey
        );
        let epoch_key = IbeEpochKeys::<T>::get(key.epoch).ok_or(Error::<T>::UnknownIbeEpoch)?;
        ensure!(epoch_key.key_id == key.key_id, Error::<T>::WrongIbeEpochKey);
        ensure!(
            key.target_block >= epoch_key.first_block && key.target_block <= epoch_key.last_block,
            Error::<T>::IbeEpochKeyInactive
        );
        let current_block_u64: u64 =
            frame_system::Pallet::<T>::block_number().saturated_into::<u64>();
        ensure!(
            current_block_u64 >= key.target_block,
            Error::<T>::IbeKeyTooEarly
        );
        ensure!(
            key.finalized_ordering_block_number >= key.target_block,
            Error::<T>::InvalidIbeFinalityPoint
        );
        ensure!(
            IbeBlockDecryptionKeys::<T>::get(block_key_storage_key(
                key.epoch,
                key.target_block,
                key.key_id,
            ))
            .is_none(),
            Error::<T>::IbeKeyAlreadyPublished
        );
        let genesis_hash = frame_system::Pallet::<T>::block_hash(BlockNumberFor::<T>::zero());
        ensure!(
            T::IbeKeyVerifier::verify_block_identity_key(
                genesis_hash,
                &epoch_key,
                key.target_block,
                key.identity_decryption_key.as_slice(),
            ),
            Error::<T>::InvalidIbeBlockDecryptionKey
        );
        Ok(())
    }

    fn store_pending_encrypted(
        who: T::AccountId,
        encrypted_call: BoundedVec<u8, MaxEncryptedCallSize>,
    ) -> Result<u32, DispatchError> {
        let pending_count: u32 = PendingExtrinsics::<T>::count();
        let max_pending: u32 = MaxPendingExtrinsicsLimit::<T>::get();

        ensure!(
            pending_count < max_pending,
            Error::<T>::TooManyPendingExtrinsics
        );

        let index = NextPendingExtrinsicIndex::<T>::get();

        let pending = PendingExtrinsic::<T> {
            who,
            encrypted_call,
            submitted_at: frame_system::Pallet::<T>::block_number(),
        };

        PendingExtrinsics::<T>::insert(index, pending);
        NextPendingExtrinsicIndex::<T>::put(index.saturating_add(1));

        Ok(index)
    }

    /// Process pending encrypted extrinsics up to the weight limit.
    /// Returns the total weight consumed.
    pub fn process_pending_extrinsics() -> Weight {
        let next_index = NextPendingExtrinsicIndex::<T>::get();
        let count: u32 = PendingExtrinsics::<T>::count();
        let mut weight = T::DbWeight::get().reads(2);

        if count == 0 {
            return weight;
        }

        let start_index = next_index.saturating_sub(count);
        let current_block = frame_system::Pallet::<T>::block_number();

        for index in start_index..next_index {
            let Some(pending) = PendingExtrinsics::<T>::get(index) else {
                weight = weight.saturating_add(T::DbWeight::get().reads(1));
                continue;
            };

            let remove_weight = T::DbWeight::get().reads_writes(1, 2);

            if IbeEncryptedExtrinsicV1::is_v2_prefixed(pending.encrypted_call.as_slice()) {
                match Self::process_pending_ibe_extrinsic(index, pending, weight, remove_weight) {
                    PendingProcess::Continue(new_weight) => {
                        weight = new_weight;
                        continue;
                    }
                    PendingProcess::Break(new_weight) => {
                        weight = new_weight;
                        break;
                    }
                }
            }

            // Legacy/non-v2 deferred RuntimeCall queue behavior.
            // v2 entries do not expire here; missing keys postpone the queue head.
            let age = current_block.saturating_sub(pending.submitted_at);

            if age > ExtrinsicLifetime::<T>::get().into() {
                Self::remove_pending_index(index);
                weight = weight.saturating_add(remove_weight);
                Self::deposit_event(Event::ExtrinsicExpired { index });
                continue;
            }

            let Ok(call) = T::ExtrinsicDecryptor::decrypt(pending.encrypted_call.as_slice()) else {
                Self::remove_pending_index(index);
                weight = weight.saturating_add(remove_weight);
                Self::deposit_event(Event::ExtrinsicDecodeFailed { index });
                continue;
            };

            let info = call.get_dispatch_info();
            let dispatch_weight = T::DbWeight::get()
                .writes(2)
                .saturating_add(info.call_weight);

            let max_extrinsic_weight = Weight::from_parts(MaxExtrinsicWeight::<T>::get(), 0);
            if info.call_weight.any_gt(max_extrinsic_weight) {
                Self::remove_pending_index(index);
                weight = weight.saturating_add(remove_weight);

                Self::deposit_event(Event::ExtrinsicWeightExceeded { index });

                continue;
            }

            let max_weight = Weight::from_parts(OnInitializeWeight::<T>::get(), 0);

            if weight.saturating_add(dispatch_weight).any_gt(max_weight) {
                Self::deposit_event(Event::ExtrinsicPostponed { index });
                break;
            }

            Self::remove_pending_index(index);
            weight = weight.saturating_add(remove_weight);

            let origin: T::RuntimeOrigin = frame_system::RawOrigin::Signed(pending.who).into();
            let result = call.dispatch(origin);

            match result {
                Ok(post_info) => {
                    let actual_weight = post_info.actual_weight.unwrap_or(info.call_weight);
                    weight = weight.saturating_add(actual_weight);

                    Self::deposit_event(Event::ExtrinsicDispatched { index });
                }
                Err(e) => {
                    weight = weight.saturating_add(info.call_weight);

                    Self::deposit_event(Event::ExtrinsicDispatchFailed {
                        index,
                        error: e.error,
                    });
                }
            }
        }

        weight
    }

    fn process_pending_ibe_extrinsic(
        index: u32,
        pending: PendingExtrinsic<T>,
        mut weight: Weight,
        remove_weight: Weight,
    ) -> PendingProcess {
        let outcome = T::IbeEncryptedTxDecryptor::decrypt(pending.encrypted_call.as_slice());

        let inner = match outcome {
            IbeDecryptOutcome::NotReady => {
                Self::deposit_event(Event::ExtrinsicPostponed { index });
                return PendingProcess::Break(weight);
            }
            IbeDecryptOutcome::InvalidAfterKeyAvailable => {
                Self::remove_pending_index(index);
                weight = weight.saturating_add(remove_weight);

                Self::deposit_event(Event::IbeEncryptedExtrinsicInvalid { index });
                return PendingProcess::Continue(weight);
            }
            IbeDecryptOutcome::Ready(inner) => inner,
        };

        let Some(info) = T::DecryptedExtrinsicExecutor::dispatch_info(&inner) else {
            Self::remove_pending_index(index);
            weight = weight.saturating_add(remove_weight);

            Self::deposit_event(Event::IbeEncryptedExtrinsicInvalid { index });
            return PendingProcess::Continue(weight);
        };

        let dispatch_weight = T::DbWeight::get()
            .writes(2)
            .saturating_add(info.call_weight);

        let max_extrinsic_weight = Weight::from_parts(MaxExtrinsicWeight::<T>::get(), 0);

        if info.call_weight.any_gt(max_extrinsic_weight) {
            Self::remove_pending_index(index);
            weight = weight.saturating_add(remove_weight);

            Self::deposit_event(Event::ExtrinsicWeightExceeded { index });
            Self::deposit_event(Event::IbeEncryptedExtrinsicExecuted {
                index,
                success: false,
            });

            return PendingProcess::Continue(weight);
        }

        let max_weight = Weight::from_parts(OnInitializeWeight::<T>::get(), 0);

        if weight.saturating_add(dispatch_weight).any_gt(max_weight) {
            Self::deposit_event(Event::ExtrinsicPostponed { index });
            return PendingProcess::Break(weight);
        }

        Self::remove_pending_index(index);
        weight = weight.saturating_add(remove_weight);

        let applied = T::DecryptedExtrinsicExecutor::apply(inner);

        weight = weight.saturating_add(applied.consumed_weight);

        Self::deposit_event(Event::IbeEncryptedExtrinsicExecuted {
            index,
            success: applied.success,
        });

        PendingProcess::Continue(weight)
    }

    pub fn ibe_block_decryption_key(
        epoch: u64,
        target_block: u64,
        key_id: [u8; KEY_ID_LEN],
    ) -> Option<IbeBlockDecryptionKeyV1> {
        IbeBlockDecryptionKeys::<T>::get(block_key_storage_key(epoch, target_block, key_id))
    }

    pub fn pending_ibe_identities(limit: u32) -> Vec<IbePendingIdentity> {
        let limit_usize = if limit == 0 {
            usize::MAX
        } else {
            limit as usize
        };
        let next_index = NextPendingExtrinsicIndex::<T>::get();
        let count: u32 = PendingExtrinsics::<T>::count();
        let start_index = next_index.saturating_sub(count);
        let mut identities = sp_std::collections::btree_map::BTreeMap::<
            (u64, u64, [u8; KEY_ID_LEN]),
            (u32, u32),
        >::new();
        for index in start_index..next_index {
            if identities.len() >= limit_usize {
                break;
            }
            let Some(meta) = PendingIbeMetadata::<T>::get(index) else {
                continue;
            };
            let key = (meta.epoch, meta.target_block, meta.key_id);
            identities
                .entry(key)
                .and_modify(|range| {
                    range.0 = range.0.min(index);
                    range.1 = range.1.max(index);
                })
                .or_insert((index, index));
        }
        identities
            .into_iter()
            .map(
                |((epoch, target_block, key_id), (first_queue_index, last_queue_index))| {
                    IbePendingIdentity {
                        epoch,
                        target_block,
                        key_id,
                        first_queue_index,
                        last_queue_index,
                    }
                },
            )
            .collect()
    }

    pub fn pending_encrypted_queue_len() -> u32 {
        PendingExtrinsics::<T>::count()
    }

    pub fn has_ibe_block_key(epoch: u64, target_block: u64, key_id: [u8; KEY_ID_LEN]) -> bool {
        Self::ibe_block_decryption_key(epoch, target_block, key_id).is_some()
    }

    pub fn try_decode_shielded_tx<Block: BlockT, Context: Default>(
        uxt: ExtrinsicOf<Block>,
    ) -> Option<ShieldedTransaction>
    where
        Block::Extrinsic: Checkable<Context>,
        CheckedOf<Block::Extrinsic, Context>: Applyable,
        ApplyableCallOf<CheckedOf<Block::Extrinsic, Context>>: IsSubType<Call<T>>,
    {
        let encoded = uxt.encode();
        let uxt = <Block::Extrinsic as codec::DecodeLimit>::decode_all_with_depth_limit(
            MAX_EXTRINSIC_DEPTH,
            &mut &encoded[..],
        )
        .inspect_err(
            |e| log::debug!(target: LOG_TARGET, "Failed to decode shielded extrinsic: {:?}", e),
        )
        .ok()?;

        let xt = ExtrinsicOf::<Block>::check(uxt, &Context::default())
            .inspect_err(
                |e| log::debug!(target: LOG_TARGET, "Failed to check shielded extrinsic: {:?}", e),
            )
            .ok()?;
        let call = xt.call();

        let Some(Call::submit_encrypted { ciphertext }) = IsSubType::<Call<T>>::is_sub_type(call)
        else {
            return None;
        };

        // v2 envelopes are handled by the threshold-IBE queue, not by the v1
        // author-local unshielding path.
        if IbeEncryptedExtrinsicV1::is_v2_prefixed(ciphertext.as_slice()) {
            return None;
        }

        ShieldedTransaction::parse(ciphertext)
    }

    pub fn is_shielded_using_current_key(key_hash: &[u8; 16]) -> bool {
        let pending = PendingKey::<T>::get();
        let pending_hash = pending.as_ref().map(|k| twox_128(&k[..]));
        pending_hash.as_ref() == Some(key_hash)
    }

    pub fn try_unshield_tx<Block: BlockT>(
        dec_key_bytes: alloc::vec::Vec<u8>,
        shielded_tx: ShieldedTransaction,
    ) -> Option<<Block as BlockT>::Extrinsic> {
        let plaintext = unshield(&dec_key_bytes, &shielded_tx).or_else(|| {
            log::debug!(target: LOG_TARGET, "Failed to unshield transaction");
            None
        })?;

        if plaintext.is_empty() {
            return None;
        }

        ExtrinsicOf::<Block>::decode(&mut &plaintext[..])
            .inspect_err(
                |e| log::debug!(target: LOG_TARGET, "Failed to decode shielded transaction: {:?}", e),
            )
            .ok()
    }
}

pub trait FindAuthors<T: Config> {
    fn find_current_author() -> Option<T::AuthorityId>;
    fn find_next_next_author() -> Option<T::AuthorityId>;
}

impl<T: Config> FindAuthors<T> for () {
    fn find_current_author() -> Option<T::AuthorityId> {
        None
    }
    fn find_next_next_author() -> Option<T::AuthorityId> {
        None
    }
}

/// Decrypt a shielded transaction using the raw decapsulation key bytes.
///
/// Performs ML-KEM-768 decapsulation followed by XChaCha20-Poly1305 AEAD decryption.
/// Runs entirely in WASM — no host functions needed.
fn unshield(
    dec_key_bytes: &[u8],
    shielded_tx: &ShieldedTransaction,
) -> Option<alloc::vec::Vec<u8>> {
    let dec_key = DecapsulationKey::<MlKem768Params>::from_bytes(dec_key_bytes.try_into().ok()?);
    let ciphertext = Ciphertext::<MlKem768>::try_from(shielded_tx.kem_ct.as_slice()).ok()?;
    let shared_secret = dec_key.decapsulate(&ciphertext).ok()?;

    let aead = XChaCha20Poly1305::new(shared_secret.as_slice().into());
    let nonce = XNonce::from_slice(&shielded_tx.nonce);
    aead.decrypt(
        nonce,
        Payload {
            msg: &shielded_tx.aead_ct,
            aad: &[],
        },
    )
    .ok()
}
