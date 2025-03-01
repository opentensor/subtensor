#![cfg_attr(not(feature = "std"), no_std)]

mod benchmarking;
#[cfg(test)]
mod tests;

pub mod types;
pub mod weights;

pub use pallet::*;
use subtensor_macros::freeze_struct;
pub use types::*;
pub use weights::WeightInfo;

use ark_serialize::CanonicalDeserialize;
use frame_support::{BoundedVec, traits::Currency};
use frame_system::pallet_prelude::BlockNumberFor;
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
        /// Account is trying to commit data too fast, rate limit exceeded
        CommitmentSetRateLimitExceeded,
    }

    #[pallet::type_value]
    /// Default value for commitment rate limit.
    pub fn DefaultRateLimit<T: Config>() -> BlockNumberFor<T> {
        T::DefaultRateLimit::get()
    }

    /// The rate limit for commitments
    #[pallet::storage]
    pub type RateLimit<T> = StorageValue<_, BlockNumberFor<T>, ValueQuery, DefaultRateLimit<T>>;

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
        RevealedData<BalanceOf<T>, T::MaxFields, BlockNumberFor<T>>,
        OptionQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set the commitment for a given netuid
        #[pallet::call_index(0)]
        #[pallet::weight((
			<T as pallet::Config>::WeightInfo::set_commitment(),
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
            if let Some(last_commit) = <LastCommitment<T>>::get(netuid, &who) {
                ensure!(
                    cur_block >= last_commit.saturating_add(RateLimit::<T>::get()),
                    Error::<T>::CommitmentSetRateLimitExceeded
                );
            }

            let fd = <BalanceOf<T>>::from(extra_fields).saturating_mul(T::FieldDeposit::get());
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
            id.deposit = T::InitialDeposit::get().saturating_add(fd);
            if id.deposit > old_deposit {
                T::Currency::reserve(&who, id.deposit.saturating_sub(old_deposit))?;
            }
            if old_deposit > id.deposit {
                let err_amount =
                    T::Currency::unreserve(&who, old_deposit.saturating_sub(id.deposit));
                debug_assert!(err_amount.is_zero());
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
                    who,
                    reveal_round: *reveal_round,
                });
            } else {
                Self::deposit_event(Event::Commitment { netuid, who });
            }

            Ok(())
        }

        /// Sudo-set the commitment rate limit
        #[pallet::call_index(1)]
        #[pallet::weight((
            <T as pallet::Config>::WeightInfo::set_rate_limit(),
			DispatchClass::Operational,
			Pays::No
		))]
        pub fn set_rate_limit(origin: OriginFor<T>, rate_limit_blocks: u32) -> DispatchResult {
            ensure_root(origin)?;
            RateLimit::<T>::set(rate_limit_blocks.into());
            Ok(())
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
    pub fn reveal_timelocked_commitments(current_block: u64) -> DispatchResult {
        let current_block = current_block
            .try_into()
            .map_err(|_| "Failed to convert u64 to BlockNumberFor<T>")?;

        for (netuid, who, mut registration) in <CommitmentOf<T>>::iter() {
            if let Some(Data::TimelockEncrypted {
                encrypted,
                reveal_round,
                ..
            }) = registration
                .info
                .fields
                .clone()
                .iter()
                .find(|data| matches!(data, Data::TimelockEncrypted { .. }))
            {
                // Calculate reveal block
                let reveal_block = Self::calculate_reveal_block(*reveal_round, registration.block)?;

                // Check if the current block has reached or exceeded the reveal block
                if current_block >= reveal_block {
                    // Deserialize the encrypted commitment into a TLECiphertext
                    let reader = &mut &encrypted[..];
                    let commit = TLECiphertext::<TinyBLS381>::deserialize_compressed(reader)
                        .map_err(|e| {
                            log::warn!("Failed to deserialize TLECiphertext for {:?}: {:?}", who, e)
                        })
                        .ok();

                    let commit = match commit {
                        Some(c) => c,
                        None => continue,
                    };

                    // Get the drand pulse for the reveal round
                    let pulse = match pallet_drand::Pulses::<T>::get(*reveal_round) {
                        Some(p) => p,
                        None => {
                            log::warn!(
                                "Failed to reveal commit for subnet {} by {:?}: missing drand round {}",
                                netuid,
                                who,
                                reveal_round
                            );
                            continue;
                        }
                    };

                    // Prepare the signature bytes
                    let signature_bytes = pulse
                        .signature
                        .strip_prefix(b"0x")
                        .unwrap_or(&pulse.signature);
                    let sig_reader = &mut &signature_bytes[..];
                    let sig = <TinyBLS381 as EngineBLS>::SignatureGroup::deserialize_compressed(
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

                    let sig = match sig {
                        Some(s) => s,
                        None => continue,
                    };

                    // Decrypt the timelock commitment
                    let decrypted_bytes: Vec<u8> =
                        tld::<TinyBLS381, AESGCMStreamCipherProvider>(commit, sig)
                            .map_err(|e| {
                                log::warn!("Failed to decrypt timelock for {:?}: {:?}", who, e)
                            })
                            .ok()
                            .unwrap_or_default();

                    if decrypted_bytes.is_empty() {
                        continue;
                    }

                    // Decode the decrypted bytes into CommitmentInfo (assuming it’s SCALE-encoded CommitmentInfo)
                    let mut reader = &decrypted_bytes[..];
                    let revealed_info: CommitmentInfo<T::MaxFields> = Decode::decode(&mut reader)
                        .map_err(|e| {
                            log::warn!("Failed to decode decrypted data for {:?}: {:?}", who, e)
                        })
                        .ok()
                        .unwrap_or_else(|| CommitmentInfo {
                            fields: BoundedVec::default(),
                        });

                    // Create RevealedData for storage
                    let revealed_data = RevealedData {
                        info: revealed_info,
                        revealed_block: current_block,
                        deposit: registration.deposit,
                    };

                    // Store the revealed data in RevealedCommitments
                    <RevealedCommitments<T>>::insert(netuid, &who, revealed_data);

                    // Remove the TimelockEncrypted field from the original commitment
                    let filtered_fields: Vec<Data> = registration.info.fields.into_iter()
                        .filter(|data| !matches!(data, Data::TimelockEncrypted { reveal_round: r, .. } if r == reveal_round))
                        .collect();
                    registration.info.fields = BoundedVec::try_from(filtered_fields)
                        .map_err(|_| "Failed to filter timelock fields")?;

                    Self::deposit_event(Event::CommitmentRevealed { netuid, who });
                }
            }
        }

        Ok(())
    }

    fn calculate_reveal_block(
        reveal_round: u64,
        commit_block: BlockNumberFor<T>,
    ) -> Result<BlockNumberFor<T>, &'static str> {
        let last_drand_round = pallet_drand::LastStoredRound::<T>::get();
        let blocks_per_round = 12_u64.checked_div(3).unwrap_or(0); // 4 blocks per round (12s blocktime / 3s round)
        let rounds_since_last = reveal_round.saturating_sub(last_drand_round);
        let blocks_to_reveal = rounds_since_last.saturating_mul(blocks_per_round);
        let reveal_block = commit_block.saturating_add(
            blocks_to_reveal
                .try_into()
                .map_err(|_| "Block number conversion failed")?,
        );
        Ok(reveal_block)
    }
}
