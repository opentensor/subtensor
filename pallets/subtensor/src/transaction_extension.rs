use crate::{
    BalancesCall, Call, ColdkeySwapScheduled, Config, CustomTransactionError, Error, Pallet,
};
use codec::{Decode, DecodeWithMemTracking, Encode};
use frame_support::dispatch::{DispatchInfo, PostDispatchInfo};
use frame_support::pallet_prelude::Weight;
use frame_support::traits::IsSubType;
use scale_info::TypeInfo;
use sp_runtime::traits::{
    AsSystemOriginSigner, DispatchInfoOf, Dispatchable, Implication, TransactionExtension,
    ValidateResult,
};
use sp_runtime::transaction_validity::{
    TransactionSource, TransactionValidity, TransactionValidityError, ValidTransaction,
};
use sp_std::marker::PhantomData;
use sp_std::vec::Vec;
use subtensor_macros::freeze_struct;
use subtensor_runtime_common::NetUid;

#[freeze_struct("2e02eb32e5cb25d3")]
#[derive(Default, Encode, Decode, DecodeWithMemTracking, Clone, Eq, PartialEq, TypeInfo)]
pub struct SubtensorTransactionExtension<T: Config + Send + Sync + TypeInfo>(pub PhantomData<T>);

impl<T: Config + Send + Sync + TypeInfo> sp_std::fmt::Debug for SubtensorTransactionExtension<T> {
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "SubtensorTransactionExtension")
    }
}

impl<T: Config + Send + Sync + TypeInfo> SubtensorTransactionExtension<T>
where
    <T as frame_system::Config>::RuntimeCall:
        Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
{
    pub fn new() -> Self {
        Self(Default::default())
    }
    pub fn validity_ok(priority: u64) -> ValidTransaction {
        ValidTransaction {
            priority,
            ..Default::default()
        }
    }

    pub fn check_weights_min_stake(who: &T::AccountId, netuid: NetUid) -> bool {
        Pallet::<T>::check_weights_min_stake(who, netuid)
    }

    pub fn result_to_validity(result: Result<(), Error<T>>, priority: u64) -> TransactionValidity {
        if let Err(err) = result {
            Err(match err {
                Error::<T>::AmountTooLow => CustomTransactionError::StakeAmountTooLow.into(),
                Error::<T>::SubnetNotExists => CustomTransactionError::SubnetDoesntExist.into(),
                Error::<T>::NotEnoughBalanceToStake => CustomTransactionError::BalanceTooLow.into(),
                Error::<T>::HotKeyAccountNotExists => {
                    CustomTransactionError::HotkeyAccountDoesntExist.into()
                }
                Error::<T>::NotEnoughStakeToWithdraw => {
                    CustomTransactionError::NotEnoughStakeToWithdraw.into()
                }
                Error::<T>::InsufficientLiquidity => {
                    CustomTransactionError::InsufficientLiquidity.into()
                }
                Error::<T>::SlippageTooHigh => CustomTransactionError::SlippageTooHigh.into(),
                Error::<T>::TransferDisallowed => CustomTransactionError::TransferDisallowed.into(),
                Error::<T>::HotKeyNotRegisteredInNetwork => {
                    CustomTransactionError::HotKeyNotRegisteredInNetwork.into()
                }
                Error::<T>::InvalidIpAddress => CustomTransactionError::InvalidIpAddress.into(),
                Error::<T>::ServingRateLimitExceeded => {
                    CustomTransactionError::ServingRateLimitExceeded.into()
                }
                Error::<T>::InvalidPort => CustomTransactionError::InvalidPort.into(),
                _ => CustomTransactionError::BadRequest.into(),
            })
        } else {
            Ok(ValidTransaction {
                priority,
                ..Default::default()
            })
        }
    }
}

impl<T: Config + Send + Sync + TypeInfo + pallet_balances::Config>
    TransactionExtension<<T as frame_system::Config>::RuntimeCall>
    for SubtensorTransactionExtension<T>
where
    <T as frame_system::Config>::RuntimeCall:
        Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <T as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<T::AccountId> + Clone,
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
    <T as frame_system::Config>::RuntimeCall: IsSubType<BalancesCall<T>>,
{
    const IDENTIFIER: &'static str = "SubtensorTransactionExtension";

    type Implicit = ();
    type Val = Option<T::AccountId>;
    type Pre = ();

    fn weight(&self, _call: &<T as frame_system::Config>::RuntimeCall) -> Weight {
        // TODO: benchmark transaction extension
        Weight::zero()
    }

    fn validate(
        &self,
        origin: <T as frame_system::Config>::RuntimeOrigin,
        call: &<T as frame_system::Config>::RuntimeCall,
        _info: &DispatchInfoOf<<T as frame_system::Config>::RuntimeCall>,
        _len: usize,
        _self_implicit: Self::Implicit,
        _inherited_implication: &impl Implication,
        _source: TransactionSource,
    ) -> ValidateResult<Self::Val, <T as frame_system::Config>::RuntimeCall> {
        // Ensure the transaction is signed, else we just skip the extension.
        let Some(who) = origin.as_system_origin_signer() else {
            return Ok((Default::default(), None, origin));
        };

        // Verify ColdkeySwapScheduled map for coldkey
        match call.is_sub_type() {
            // Whitelist
            Some(Call::schedule_swap_coldkey { .. }) => {}
            _ => {
                if ColdkeySwapScheduled::<T>::contains_key(who) {
                    return Err(CustomTransactionError::ColdkeyInSwapSchedule.into());
                }
            }
        }
        match call.is_sub_type() {
            Some(Call::commit_weights { netuid, .. }) => {
                if Self::check_weights_min_stake(who, *netuid) {
                    Ok((Default::default(), Some(who.clone()), origin))
                } else {
                    Err(CustomTransactionError::StakeAmountTooLow.into())
                }
            }
            Some(Call::reveal_weights {
                netuid,
                uids,
                values,
                salt,
                version_key,
            }) => {
                if Self::check_weights_min_stake(who, *netuid) {
                    let provided_hash = Pallet::<T>::get_commit_hash(
                        who,
                        *netuid,
                        uids,
                        values,
                        salt,
                        *version_key,
                    );
                    match Pallet::<T>::find_commit_block_via_hash(provided_hash) {
                        Some(commit_block) => {
                            if Pallet::<T>::is_reveal_block_range(*netuid, commit_block) {
                                Ok((Default::default(), Some(who.clone()), origin))
                            } else {
                                Err(CustomTransactionError::CommitBlockNotInRevealRange.into())
                            }
                        }
                        None => Err(CustomTransactionError::CommitNotFound.into()),
                    }
                } else {
                    Err(CustomTransactionError::StakeAmountTooLow.into())
                }
            }
            Some(Call::batch_reveal_weights {
                netuid,
                uids_list,
                values_list,
                salts_list,
                version_keys,
            }) => {
                if Self::check_weights_min_stake(who, *netuid) {
                    let num_reveals = uids_list.len();
                    if num_reveals == values_list.len()
                        && num_reveals == salts_list.len()
                        && num_reveals == version_keys.len()
                    {
                        let provided_hashes = (0..num_reveals)
                            .map(|i| {
                                Pallet::<T>::get_commit_hash(
                                    who,
                                    *netuid,
                                    uids_list.get(i).unwrap_or(&Vec::new()),
                                    values_list.get(i).unwrap_or(&Vec::new()),
                                    salts_list.get(i).unwrap_or(&Vec::new()),
                                    *version_keys.get(i).unwrap_or(&0_u64),
                                )
                            })
                            .collect::<Vec<_>>();

                        let batch_reveal_block = provided_hashes
                            .iter()
                            .filter_map(|hash| Pallet::<T>::find_commit_block_via_hash(*hash))
                            .collect::<Vec<_>>();

                        if provided_hashes.len() == batch_reveal_block.len() {
                            if Pallet::<T>::is_batch_reveal_block_range(*netuid, batch_reveal_block)
                            {
                                Ok((Default::default(), Some(who.clone()), origin))
                            } else {
                                Err(CustomTransactionError::CommitBlockNotInRevealRange.into())
                            }
                        } else {
                            Err(CustomTransactionError::CommitNotFound.into())
                        }
                    } else {
                        Err(CustomTransactionError::InputLengthsUnequal.into())
                    }
                } else {
                    Err(CustomTransactionError::StakeAmountTooLow.into())
                }
            }
            Some(Call::set_weights { netuid, .. }) => {
                if Self::check_weights_min_stake(who, *netuid) {
                    Ok((Default::default(), Some(who.clone()), origin))
                } else {
                    Err(CustomTransactionError::StakeAmountTooLow.into())
                }
            }
            Some(Call::commit_timelocked_weights {
                netuid,
                reveal_round,
                ..
            }) => {
                if Self::check_weights_min_stake(who, *netuid) {
                    if *reveal_round < pallet_drand::LastStoredRound::<T>::get() {
                        return Err(CustomTransactionError::InvalidRevealRound.into());
                    }
                    Ok((Default::default(), Some(who.clone()), origin))
                } else {
                    Err(CustomTransactionError::StakeAmountTooLow.into())
                }
            }
            Some(Call::register { netuid, .. } | Call::burned_register { netuid, .. }) => {
                let registrations_this_interval =
                    Pallet::<T>::get_registrations_this_interval(*netuid);
                let max_registrations_per_interval =
                    Pallet::<T>::get_target_registrations_per_interval(*netuid);
                if registrations_this_interval >= (max_registrations_per_interval.saturating_mul(3))
                {
                    // If the registration limit for the interval is exceeded, reject the transaction
                    return Err(CustomTransactionError::RateLimitExceeded.into());
                }

                Ok((Default::default(), Some(who.clone()), origin))
            }
            Some(Call::serve_axon {
                netuid,
                version,
                ip,
                port,
                ip_type,
                protocol,
                placeholder1,
                placeholder2,
            }) => {
                // Fully validate the user input
                Self::result_to_validity(
                    Pallet::<T>::validate_serve_axon(
                        who,
                        *netuid,
                        *version,
                        *ip,
                        *port,
                        *ip_type,
                        *protocol,
                        *placeholder1,
                        *placeholder2,
                    ),
                    0u64,
                )
                .map(|validity| (validity, Some(who.clone()), origin.clone()))
            }
            _ => Ok((Default::default(), Some(who.clone()), origin)),
        }
    }

    // NOTE: Add later when we put in a pre and post dispatch step.
    fn prepare(
        self,
        _val: Self::Val,
        _origin: &<T as frame_system::Config>::RuntimeOrigin,
        _call: &<T as frame_system::Config>::RuntimeCall,
        _info: &DispatchInfoOf<<T as frame_system::Config>::RuntimeCall>,
        _len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        Ok(())
    }
}
