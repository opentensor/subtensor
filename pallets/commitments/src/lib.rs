#![cfg_attr(not(feature = "std"), no_std)]

mod benchmarking;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

pub mod types;
pub mod weights;

pub use pallet::*;
use subtensor_macros::freeze_struct;
pub use types::*;
pub use weights::WeightInfo;

use ark_serialize::CanonicalDeserialize;
use frame_support::{BoundedVec, traits::Currency};
use scale_info::prelude::collections::BTreeSet;
use sp_runtime::SaturatedConversion;
use sp_runtime::{Saturating, traits::Zero};
use sp_std::{boxed::Box, vec::Vec};
use tle::{
    curves::drand::TinyBLS381,
    stream_ciphers::AESGCMStreamCipherProvider,
    tlock::{TLECiphertext, tld},
};
use w3f_bls::EngineBLS;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
#[deny(missing_docs)]
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, traits::ReservableCurrency};
    use frame_system::pallet_prelude::{BlockNumberFor, *};

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_drand::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Currency type that will be used to place deposits on neurons
        type Currency: ReservableCurrency<Self::AccountId> + Send + Sync;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// Interface to access-limit metadata commitments
        type CanCommit: CanCommit<Self::AccountId>;

        /// The maximum number of additional fields that can be added to a commitment
        #[pallet::constant]
        type MaxFields: Get<u32> + TypeInfo + 'static;

        /// The amount held on deposit for a registered identity
        #[pallet::constant]
        type InitialDeposit: Get<BalanceOf<Self>>;

        /// The amount held on deposit per additional field for a registered identity.
        #[pallet::constant]
        type FieldDeposit: Get<BalanceOf<Self>>;

        /// The rate limit for commitments
        #[pallet::constant]
        type DefaultRateLimit: Get<BlockNumberFor<Self>>;

        /// Used to retreive the given subnet's tempo
        type TempoInterface: GetTempoInterface;
    }

    /// Used to retreive the given subnet's tempo  
    pub trait GetTempoInterface {
        /// Used to retreive the epoch index for the given subnet.
        fn get_epoch_index(netuid: u16, cur_block: u64) -> u64;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A commitment was set
        Commitment {
            /// The netuid of the commitment
            netuid: u16,
            /// The account
            who: T::AccountId,
        },
        /// A timelock-encrypted commitment was set
        TimelockCommitment {
            /// The netuid of the commitment
            netuid: u16,
            /// The account
            who: T::AccountId,
            /// The drand round to reveal
            reveal_round: u64,
        },
        /// A timelock-encrypted commitment was auto-revealed
        CommitmentRevealed {
            /// The netuid of the commitment
            netuid: u16,
            /// The account
            who: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Account passed too many additional fields to their commitment
        TooManyFieldsInCommitmentInfo,
        /// Account is not allow to make commitments to the chain
        AccountNotAllowedCommit,
        /// Space Limit Exceeded for the current interval
        SpaceLimitExceeded,
        /// Indicates that unreserve returned a leftover, which is unexpected.
        UnexpectedUnreserveLeftover,
    }

    /// Tracks all CommitmentOf that have at least one timelocked field.
    #[pallet::storage]
    #[pallet::getter(fn timelocked_index)]
    pub type TimelockedIndex<T: Config> =
        StorageValue<_, BTreeSet<(u16, T::AccountId)>, ValueQuery>;

    /// Identity data by account
    #[pallet::storage]
    #[pallet::getter(fn commitment_of)]
    pub(super) type CommitmentOf<T: Config> = StorageDoubleMap<
        _,
        Identity,
        u16,
        Twox64Concat,
        T::AccountId,
        Registration<BalanceOf<T>, T::MaxFields, BlockNumberFor<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn last_commitment)]
    pub(super) type LastCommitment<T: Config> = StorageDoubleMap<
        _,
        Identity,
        u16,
        Twox64Concat,
        T::AccountId,
        BlockNumberFor<T>,
        OptionQuery,
    >;
    #[pallet::storage]
    #[pallet::getter(fn revealed_commitments)]
    pub(super) type RevealedCommitments<T: Config> = StorageDoubleMap<
        _,
        Identity,
        u16,
        Twox64Concat,
        T::AccountId,
        Vec<(Vec<u8>, u64)>, // Reveals<(Data, RevealBlock)>
        OptionQuery,
    >;

    /// Maps (netuid, who) -> usage (how many “bytes” they've committed)
    /// in the RateLimit window
    #[pallet::storage]
    #[pallet::getter(fn used_space_of)]
    pub type UsedSpaceOf<T: Config> =
        StorageDoubleMap<_, Identity, u16, Twox64Concat, T::AccountId, UsageTracker, OptionQuery>;

    #[pallet::type_value]
    /// The default Maximum Space
    pub fn DefaultMaxSpace() -> u32 {
        3100
    }

    #[pallet::storage]
    #[pallet::getter(fn max_space_per_user_per_rate_limit)]
    pub type MaxSpace<T> = StorageValue<_, u32, ValueQuery, DefaultMaxSpace>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set the commitment for a given netuid
        #[pallet::call_index(0)]
        #[pallet::weight((
            Weight::from_parts(38_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64)),
            DispatchClass::Operational,
            Pays::No
        ))]
        pub fn set_commitment(
            origin: OriginFor<T>,
            netuid: u16,
            info: Box<CommitmentInfo<T::MaxFields>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                T::CanCommit::can_commit(netuid, &who),
                Error::<T>::AccountNotAllowedCommit
            );

            let extra_fields = info.fields.len() as u32;
            ensure!(
                extra_fields <= T::MaxFields::get(),
                Error::<T>::TooManyFieldsInCommitmentInfo
            );

            let cur_block = <frame_system::Pallet<T>>::block_number();

            let min_used_space: u64 = 100;
            let required_space: u64 = info
                .fields
                .iter()
                .map(|field| field.len_for_rate_limit())
                .sum::<u64>()
                .max(min_used_space);

            let mut usage = UsedSpaceOf::<T>::get(netuid, &who).unwrap_or_default();
            let cur_block_u64 = cur_block.saturated_into::<u64>();
            let current_epoch = T::TempoInterface::get_epoch_index(netuid, cur_block_u64);

            if usage.last_epoch != current_epoch {
                usage.last_epoch = current_epoch;
                usage.used_space = 0;
            }

            let max_allowed = MaxSpace::<T>::get() as u64;
            ensure!(
                usage.used_space.saturating_add(required_space) <= max_allowed,
                Error::<T>::SpaceLimitExceeded
            );

            usage.used_space = usage.used_space.saturating_add(required_space);

            UsedSpaceOf::<T>::insert(netuid, &who, usage);

            let mut id = match <CommitmentOf<T>>::get(netuid, &who) {
                Some(mut id) => {
                    id.info = *info.clone();
                    id.block = cur_block;
                    id
                }
                None => Registration {
                    info: *info.clone(),
                    block: cur_block,
                    deposit: Zero::zero(),
                },
            };

            let old_deposit = id.deposit;
            let fd = <BalanceOf<T>>::from(extra_fields).saturating_mul(T::FieldDeposit::get());
            id.deposit = T::InitialDeposit::get().saturating_add(fd);
            if id.deposit > old_deposit {
                T::Currency::reserve(&who, id.deposit.saturating_sub(old_deposit))?;
            }
            if old_deposit > id.deposit {
                let err_amount =
                    T::Currency::unreserve(&who, old_deposit.saturating_sub(id.deposit));
                if !err_amount.is_zero() {
                    return Err(Error::<T>::UnexpectedUnreserveLeftover.into());
                }
            }

            <CommitmentOf<T>>::insert(netuid, &who, id);
            <LastCommitment<T>>::insert(netuid, &who, cur_block);

            if let Some(Data::TimelockEncrypted { reveal_round, .. }) = info
                .fields
                .iter()
                .find(|data| matches!(data, Data::TimelockEncrypted { .. }))
            {
                Self::deposit_event(Event::TimelockCommitment {
                    netuid,
                    who: who.clone(),
                    reveal_round: *reveal_round,
                });

                TimelockedIndex::<T>::mutate(|index| {
                    index.insert((netuid, who.clone()));
                });
            } else {
                Self::deposit_event(Event::Commitment {
                    netuid,
                    who: who.clone(),
                });

                TimelockedIndex::<T>::mutate(|index| {
                    index.remove(&(netuid, who.clone()));
                });
            }

            Ok(())
        }

        /// Sudo-set the commitment rate limit
        // #[pallet::call_index(1)]
        // #[pallet::weight((
        //     Weight::from_parts(3_596_000, 0)
		// 	.saturating_add(T::DbWeight::get().reads(0_u64))
		// 	.saturating_add(T::DbWeight::get().writes(1_u64)),
		// 	DispatchClass::Operational,
		// 	Pays::No
		// ))]
        // pub fn set_rate_limit(origin: OriginFor<T>, rate_limit_blocks: u32) -> DispatchResult {
        //     ensure_root(origin)?;
        //     RateLimit::<T>::set(rate_limit_blocks.into());
        //     Ok(())
        // }

        /// Sudo-set MaxSpace
        #[pallet::call_index(2)]
        #[pallet::weight((
            Weight::from_parts(3_556_000, 0)
			.saturating_add(T::DbWeight::get().reads(0_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64)),
            DispatchClass::Operational,
            Pays::No
        ))]
        pub fn set_max_space(origin: OriginFor<T>, new_limit: u32) -> DispatchResult {
            ensure_root(origin)?;
            MaxSpace::<T>::set(new_limit);
            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            if let Err(e) = Self::reveal_timelocked_commitments() {
                log::debug!(
                    "Failed to unveil matured commitments on block {:?}: {:?}",
                    n,
                    e
                );
            }
            Weight::from_parts(0, 0)
        }
    }
}

// Interfaces to interact with other pallets
pub trait CanCommit<AccountId> {
    fn can_commit(netuid: u16, who: &AccountId) -> bool;
}

impl<A> CanCommit<A> for () {
    fn can_commit(_: u16, _: &A) -> bool {
        false
    }
}

/************************************************************
    CallType definition
************************************************************/
#[derive(Debug, PartialEq, Default)]
pub enum CallType {
    SetCommitment,
    #[default]
    Other,
}

use {
    frame_support::{
        dispatch::{DispatchInfo, DispatchResult, PostDispatchInfo},
        pallet_prelude::{Decode, Encode, PhantomData, TypeInfo},
        traits::IsSubType,
    },
    sp_runtime::{
        traits::{DispatchInfoOf, Dispatchable, PostDispatchInfoOf, SignedExtension},
        transaction_validity::{TransactionValidity, TransactionValidityError, ValidTransaction},
    },
};

#[freeze_struct("6a00398e14a8a984")]
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
pub struct CommitmentsSignedExtension<T: Config + Send + Sync + TypeInfo>(pub PhantomData<T>);

impl<T: Config + Send + Sync + TypeInfo> Default for CommitmentsSignedExtension<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Config + Send + Sync + TypeInfo> CommitmentsSignedExtension<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
{
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn get_priority_vanilla() -> u64 {
        // Return high priority so that every extrinsic except set_weights function will
        // have a higher priority than the set_weights call
        u64::MAX
    }
}

impl<T: Config + Send + Sync + TypeInfo> sp_std::fmt::Debug for CommitmentsSignedExtension<T> {
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "SignedExtension")
    }
}

impl<T: Config + Send + Sync + TypeInfo> SignedExtension for CommitmentsSignedExtension<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
{
    const IDENTIFIER: &'static str = "CommitmentsSignedExtension";

    type AccountId = T::AccountId;
    type Call = T::RuntimeCall;
    type AdditionalSigned = ();
    type Pre = (CallType, u64, Self::AccountId);

    fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
        Ok(())
    }

    fn validate(
        &self,
        _: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        call.is_sub_type();
        Ok(ValidTransaction {
            priority: Self::get_priority_vanilla(),
            ..Default::default()
        })
    }

    // NOTE: Add later when we put in a pre and post dispatch step.
    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        match call.is_sub_type() {
            Some(Call::set_commitment { .. }) => {
                let transaction_fee = 0;
                Ok((CallType::SetCommitment, transaction_fee, who.clone()))
            }
            _ => {
                let transaction_fee = 0;
                Ok((CallType::Other, transaction_fee, who.clone()))
            }
        }
    }

    fn post_dispatch(
        _maybe_pre: Option<Self::Pre>,
        _info: &DispatchInfoOf<Self::Call>,
        _post_info: &PostDispatchInfoOf<Self::Call>,
        _len: usize,
        _result: &DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        Ok(())
    }
}

impl<T: Config> Pallet<T> {
    pub fn reveal_timelocked_commitments() -> DispatchResult {
        let index = TimelockedIndex::<T>::get();
        for (netuid, who) in index.clone() {
            let Some(mut registration) = <CommitmentOf<T>>::get(netuid, &who) else {
                TimelockedIndex::<T>::mutate(|idx| {
                    idx.remove(&(netuid, who.clone()));
                });
                continue;
            };

            let original_fields = registration.info.fields.clone();
            let mut remain_fields = Vec::new();
            let mut revealed_fields = Vec::new();

            for data in original_fields {
                match data {
                    Data::TimelockEncrypted {
                        encrypted,
                        reveal_round,
                    } => {
                        let pulse = match pallet_drand::Pulses::<T>::get(reveal_round) {
                            Some(p) => p,
                            None => {
                                remain_fields.push(Data::TimelockEncrypted {
                                    encrypted,
                                    reveal_round,
                                });
                                continue;
                            }
                        };

                        let signature_bytes = pulse
                            .signature
                            .strip_prefix(b"0x")
                            .unwrap_or(&pulse.signature);
                        let sig_reader = &mut &signature_bytes[..];
                        let sig =
                            <TinyBLS381 as EngineBLS>::SignatureGroup::deserialize_compressed(
                                sig_reader,
                            )
                            .map_err(|e| {
                                log::warn!(
                                    "Failed to deserialize drand signature for {:?}: {:?}",
                                    who,
                                    e
                                )
                            })
                            .ok();

                        let Some(sig) = sig else {
                            log::warn!("No sig after deserialization");
                            continue;
                        };

                        let reader = &mut &encrypted[..];
                        let commit = TLECiphertext::<TinyBLS381>::deserialize_compressed(reader)
                            .map_err(|e| {
                                log::warn!(
                                    "Failed to deserialize TLECiphertext for {:?}: {:?}",
                                    who,
                                    e
                                )
                            })
                            .ok();

                        let Some(commit) = commit else {
                            log::warn!("No commit after deserialization");
                            continue;
                        };

                        let decrypted_bytes: Vec<u8> =
                            tld::<TinyBLS381, AESGCMStreamCipherProvider>(commit, sig)
                                .map_err(|e| {
                                    log::warn!("Failed to decrypt timelock for {:?}: {:?}", who, e)
                                })
                                .ok()
                                .unwrap_or_default();

                        if decrypted_bytes.is_empty() {
                            log::warn!("Bytes were decrypted for {:?} but they are empty", who);
                            continue;
                        }

                        revealed_fields.push(decrypted_bytes);
                    }

                    other => remain_fields.push(other),
                }
            }

            if !revealed_fields.is_empty() {
                let mut existing_reveals =
                    RevealedCommitments::<T>::get(netuid, &who).unwrap_or_default();

                let current_block = <frame_system::Pallet<T>>::block_number();
                let block_u64 = current_block.saturated_into::<u64>();

                // Push newly revealed items onto the tail of existing_reveals and emit the event
                for revealed_bytes in revealed_fields {
                    existing_reveals.push((revealed_bytes, block_u64));

                    Self::deposit_event(Event::CommitmentRevealed {
                        netuid,
                        who: who.clone(),
                    });
                }

                const MAX_REVEALS: usize = 10;
                if existing_reveals.len() > MAX_REVEALS {
                    let remove_count = existing_reveals.len().saturating_sub(MAX_REVEALS);
                    existing_reveals.drain(0..remove_count);
                }

                RevealedCommitments::<T>::insert(netuid, &who, existing_reveals);
            }

            registration.info.fields = BoundedVec::try_from(remain_fields)
                .map_err(|_| "Failed to build BoundedVec for remain_fields")?;

            match registration.info.fields.is_empty() {
                true => {
                    <CommitmentOf<T>>::remove(netuid, &who);
                    TimelockedIndex::<T>::mutate(|idx| {
                        idx.remove(&(netuid, who.clone()));
                    });
                }
                false => {
                    <CommitmentOf<T>>::insert(netuid, &who, &registration);
                    let has_timelock = registration
                        .info
                        .fields
                        .iter()
                        .any(|f| matches!(f, Data::TimelockEncrypted { .. }));
                    if !has_timelock {
                        TimelockedIndex::<T>::mutate(|idx| {
                            idx.remove(&(netuid, who.clone()));
                        });
                    }
                }
            }
        }

        Ok(())
    }
}
