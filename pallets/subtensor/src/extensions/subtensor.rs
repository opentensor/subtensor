use crate::{BalancesCall, Call, CheckColdkeySwap, Config, Error, Pallet, TransactionType};
use codec::{Decode, DecodeWithMemTracking, Encode};
use frame_support::dispatch::{DispatchInfo, PostDispatchInfo};
use frame_support::traits::IsSubType;
use scale_info::TypeInfo;
use sp_runtime::traits::{
    AsSystemOriginSigner, DispatchInfoOf, Dispatchable, Implication, TransactionExtension,
    ValidateResult,
};
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidityError};
use sp_runtime::{
    impl_tx_ext_default,
    transaction_validity::{TransactionSource, TransactionValidity, ValidTransaction},
};
use sp_std::marker::PhantomData;
use sp_std::vec::Vec;
use subtensor_macros::freeze_struct;
use subtensor_runtime_common::{CustomTransactionError, NetUid, NetUidStorageIndex};

const ADD_STAKE_BURN_PRIORITY_BOOST: u64 = 100;

type CallOf<T> = <T as frame_system::Config>::RuntimeCall;
type OriginOf<T> = <T as frame_system::Config>::RuntimeOrigin;

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
    CallOf<T>: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo> + IsSubType<Call<T>>,
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
                Error::<T>::AmountTooLow => CustomTransactionError::StakeAmountTooLow,
                Error::<T>::SubnetNotExists => CustomTransactionError::SubnetNotExists,
                Error::<T>::NotEnoughBalanceToStake => CustomTransactionError::BalanceTooLow,
                Error::<T>::HotKeyAccountNotExists => {
                    CustomTransactionError::HotkeyAccountDoesntExist
                }
                Error::<T>::NotEnoughStakeToWithdraw => {
                    CustomTransactionError::NotEnoughStakeToWithdraw
                }
                Error::<T>::InsufficientLiquidity => CustomTransactionError::InsufficientLiquidity,
                Error::<T>::SlippageTooHigh => CustomTransactionError::SlippageTooHigh,
                Error::<T>::TransferDisallowed => CustomTransactionError::TransferDisallowed,
                Error::<T>::HotKeyNotRegisteredInNetwork => {
                    CustomTransactionError::HotKeyNotRegisteredInNetwork
                }
                Error::<T>::InvalidIpAddress => CustomTransactionError::InvalidIpAddress,
                Error::<T>::ServingRateLimitExceeded => {
                    CustomTransactionError::ServingRateLimitExceeded
                }
                Error::<T>::InvalidPort => CustomTransactionError::InvalidPort,
                _ => CustomTransactionError::BadRequest,
            }
            .into())
        } else {
            Ok(ValidTransaction {
                priority,
                ..Default::default()
            })
        }
    }
}

impl<T> TransactionExtension<CallOf<T>> for SubtensorTransactionExtension<T>
where
    T: Config
        + Send
        + Sync
        + TypeInfo
        + pallet_balances::Config
        + pallet_subtensor_proxy::Config
        + pallet_shield::Config,
    CallOf<T>: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<Call<T>>
        + IsSubType<BalancesCall<T>>
        + IsSubType<pallet_subtensor_proxy::Call<T>>
        + IsSubType<pallet_shield::Call<T>>,
    OriginOf<T>: AsSystemOriginSigner<T::AccountId> + Clone,
{
    const IDENTIFIER: &'static str = "SubtensorTransactionExtension";

    type Implicit = ();
    type Val = ();
    type Pre = ();

    fn validate(
        &self,
        origin: OriginOf<T>,
        call: &CallOf<T>,
        _info: &DispatchInfoOf<CallOf<T>>,
        _len: usize,
        _self_implicit: Self::Implicit,
        _inherited_implication: &impl Implication,
        _source: TransactionSource,
    ) -> ValidateResult<Self::Val, CallOf<T>> {
        // Ensure the transaction is signed, else we just skip the extension.
        let Some(who) = origin.as_system_origin_signer() else {
            return Ok((Default::default(), (), origin));
        };

        // TODO: move into tx extension pipeline but require node upgrade
        CheckColdkeySwap::<T>::new().validate(
            origin.clone(),
            call,
            _info,
            _len,
            _self_implicit,
            _inherited_implication,
            _source,
        )?;

        match call.is_sub_type() {
            Some(Call::commit_weights { netuid, .. }) => {
                if Self::check_weights_min_stake(who, *netuid) {
                    Ok((Default::default(), (), origin))
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
                        NetUidStorageIndex::from(*netuid),
                        uids,
                        values,
                        salt,
                        *version_key,
                    );
                    match Pallet::<T>::find_commit_block_via_hash(provided_hash) {
                        Some(commit_block) => {
                            if Pallet::<T>::is_reveal_block_range(*netuid, commit_block) {
                                Ok((Default::default(), (), origin))
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
                                    NetUidStorageIndex::from(*netuid),
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
                                Ok((Default::default(), (), origin))
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
                    Ok((Default::default(), (), origin))
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
                    Ok((Default::default(), (), origin))
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

                Ok((Default::default(), (), origin))
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
                .map(|validity| (validity, (), origin.clone()))
            }
            Some(Call::register_network { .. }) => {
                if !TransactionType::RegisterNetwork.passes_rate_limit::<T>(who) {
                    return Err(CustomTransactionError::RateLimitExceeded.into());
                }

                Ok((Default::default(), (), origin))
            }
            Some(Call::associate_evm_key { netuid, .. }) => {
                let uid = Pallet::<T>::get_uid_for_net_and_hotkey(*netuid, who)
                    .map_err(|_| CustomTransactionError::UidNotFound)?;
                Pallet::<T>::ensure_evm_key_associate_rate_limit(*netuid, uid)
                    .map_err(|_| CustomTransactionError::EvmKeyAssociateRateLimitExceeded)?;
                Ok((Default::default(), (), origin))
            }
            Some(Call::add_stake_burn { netuid, .. }) => {
                Pallet::<T>::ensure_subnet_owner(origin.clone(), *netuid).map_err(|_| {
                    TransactionValidityError::Invalid(InvalidTransaction::BadSigner)
                })?;

                Ok((Self::validity_ok(ADD_STAKE_BURN_PRIORITY_BOOST), (), origin))
            }
            _ => Ok((Default::default(), (), origin)),
        }
    }

    impl_tx_ext_default!(CallOf<T>; weight prepare);
}
