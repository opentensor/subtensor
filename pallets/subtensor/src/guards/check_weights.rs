use crate::{Call, Config, Error, Pallet};
use frame_support::{
    dispatch::{DispatchErrorWithPostInfo, DispatchExtension, DispatchInfo, PostDispatchInfo},
    pallet_prelude::*,
    traits::{IsSubType, OriginTrait},
};
use sp_core::H256;
use sp_runtime::traits::Dispatchable;
use sp_std::{marker::PhantomData, vec::Vec};
use subtensor_runtime_common::{NetUid, NetUidStorageIndex};

type CallOf<T> = <T as frame_system::Config>::RuntimeCall;
type DispatchableOriginOf<T> = <CallOf<T> as Dispatchable>::RuntimeOrigin;
const MAX_UNREVEALED_COMMITS: u64 = 10;

/// Dispatch extension for weight-setting preconditions.
///
/// Signed weight calls are checked for batch shape, min stake, and commit/reveal
/// prerequisites before dispatch; unrelated calls and non-signed origins pass through.
pub struct CheckWeights<T: Config>(PhantomData<T>);

impl<T: Config> CheckWeights<T> {
    pub fn check(who: &T::AccountId, call: &Call<T>) -> Result<(), Error<T>> {
        Self::check_input_lengths(call)?;
        Self::check_min_stake(who, call)?;
        Self::check_commit_reveal(who, call)
    }

    fn check_input_lengths(call: &Call<T>) -> Result<(), Error<T>> {
        match call {
            Call::batch_commit_weights {
                netuids,
                commit_hashes,
            } if netuids.len() != commit_hashes.len() => Err(Error::<T>::InputLengthsUnequal),
            Call::batch_reveal_weights {
                uids_list,
                values_list,
                salts_list,
                version_keys,
                ..
            } if uids_list.len() != values_list.len()
                || uids_list.len() != salts_list.len()
                || uids_list.len() != version_keys.len() =>
            {
                Err(Error::<T>::InputLengthsUnequal)
            }
            Call::batch_set_weights {
                netuids,
                weights,
                version_keys,
            } if netuids.len() != weights.len() || netuids.len() != version_keys.len() => {
                Err(Error::<T>::InputLengthsUnequal)
            }
            _ => Ok(()),
        }
    }

    fn ensure_min_stake(who: &T::AccountId, netuid: NetUid) -> Result<(), Error<T>> {
        if Pallet::<T>::check_weights_min_stake(who, netuid) {
            Ok(())
        } else {
            Err(Error::<T>::NotEnoughStakeToSetWeights)
        }
    }

    fn check_min_stake(who: &T::AccountId, call: &Call<T>) -> Result<(), Error<T>> {
        match call {
            Call::commit_weights { netuid, .. }
            | Call::commit_mechanism_weights { netuid, .. }
            | Call::reveal_weights { netuid, .. }
            | Call::reveal_mechanism_weights { netuid, .. }
            | Call::batch_reveal_weights { netuid, .. }
            | Call::set_weights { netuid, .. }
            | Call::set_mechanism_weights { netuid, .. }
            | Call::commit_timelocked_weights { netuid, .. }
            | Call::commit_timelocked_mechanism_weights { netuid, .. }
            | Call::commit_crv3_mechanism_weights { netuid, .. } => {
                Self::ensure_min_stake(who, *netuid)
            }
            Call::batch_commit_weights { netuids, .. }
            | Call::batch_set_weights { netuids, .. } => {
                for netuid in netuids.iter() {
                    Self::ensure_min_stake(who, (*netuid).into())?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn commit_hash(
        who: &T::AccountId,
        netuid_index: NetUidStorageIndex,
        uids: &[u16],
        values: &[u16],
        salt: &[u16],
        version_key: u64,
    ) -> H256 {
        Pallet::<T>::get_commit_hash(who, netuid_index, uids, values, salt, version_key)
    }

    fn commit_epoch(hash: H256) -> Result<u64, Error<T>> {
        Pallet::<T>::find_commit_epoch_via_hash(hash).ok_or(Error::<T>::NoWeightsCommitFound)
    }

    fn check_reveal_epoch_range(netuid: NetUid, commit_epoch: u64) -> Result<(), Error<T>> {
        if Pallet::<T>::is_reveal_block_range(netuid, commit_epoch) {
            Ok(())
        } else {
            Err(Error::<T>::RevealTooEarly)
        }
    }

    fn check_reveal_hash(netuid: NetUid, hash: H256) -> Result<(), Error<T>> {
        Self::check_reveal_epoch_range(netuid, Self::commit_epoch(hash)?)
    }

    fn check_reveal(
        who: &T::AccountId,
        netuid: NetUid,
        netuid_index: NetUidStorageIndex,
        uids: &[u16],
        values: &[u16],
        salt: &[u16],
        version_key: u64,
    ) -> Result<(), Error<T>> {
        Self::check_reveal_hash(
            netuid,
            Self::commit_hash(who, netuid_index, uids, values, salt, version_key),
        )
    }

    fn check_batch_reveal(
        who: &T::AccountId,
        netuid: NetUid,
        uids_list: &[Vec<u16>],
        values_list: &[Vec<u16>],
        salts_list: &[Vec<u16>],
        version_keys: &[u64],
    ) -> Result<(), Error<T>> {
        if uids_list.len() != values_list.len()
            || uids_list.len() != salts_list.len()
            || uids_list.len() != version_keys.len()
        {
            return Err(Error::<T>::InputLengthsUnequal);
        }

        let netuid_index = NetUidStorageIndex::from(netuid);
        let commit_epochs = uids_list
            .iter()
            .zip(values_list)
            .zip(salts_list)
            .zip(version_keys)
            .map(|(((uids, values), salt), version_key)| {
                Self::commit_epoch(Self::commit_hash(
                    who,
                    netuid_index,
                    uids,
                    values,
                    salt,
                    *version_key,
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        if Pallet::<T>::is_batch_reveal_epoch_range(netuid, commit_epochs) {
            Ok(())
        } else {
            Err(Error::<T>::RevealTooEarly)
        }
    }

    fn check_commit_reveal(who: &T::AccountId, call: &Call<T>) -> Result<(), Error<T>> {
        match call {
            Call::reveal_weights {
                netuid,
                uids,
                values,
                salt,
                version_key,
            } => Self::check_reveal(
                who,
                *netuid,
                NetUidStorageIndex::from(*netuid),
                uids,
                values,
                salt,
                *version_key,
            ),
            Call::reveal_mechanism_weights {
                netuid,
                mecid,
                uids,
                values,
                salt,
                version_key,
            } => Self::check_reveal(
                who,
                *netuid,
                Pallet::<T>::get_mechanism_storage_index(*netuid, *mecid),
                uids,
                values,
                salt,
                *version_key,
            ),
            Call::batch_reveal_weights {
                netuid,
                uids_list,
                values_list,
                salts_list,
                version_keys,
            } => Self::check_batch_reveal(
                who,
                *netuid,
                uids_list,
                values_list,
                salts_list,
                version_keys,
            ),
            Call::commit_timelocked_weights { reveal_round, .. }
            | Call::commit_timelocked_mechanism_weights { reveal_round, .. }
            | Call::commit_crv3_mechanism_weights { reveal_round, .. }
                if *reveal_round < pallet_drand::LastStoredRound::<T>::get() =>
            {
                Err(Error::<T>::InvalidRevealRound)
            }
            _ => Ok(()),
        }
    }
}

impl<T> DispatchExtension<CallOf<T>> for CheckWeights<T>
where
    T: Config,
    CallOf<T>: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo> + IsSubType<Call<T>>,
    DispatchableOriginOf<T>: OriginTrait<AccountId = T::AccountId>,
{
    type Pre = ();

    fn weight(_call: &CallOf<T>) -> Weight {
        T::DbWeight::get().reads(
            1_u64
                .saturating_add(T::InitialMaxAllowedUids::get().into())
                .saturating_add(MAX_UNREVEALED_COMMITS),
        )
    }

    fn pre_dispatch(
        origin: &DispatchableOriginOf<T>,
        call: &CallOf<T>,
    ) -> Result<Self::Pre, DispatchErrorWithPostInfo> {
        let Some(who) = origin.as_signer() else {
            return Ok(());
        };

        let Some(call) = call.is_sub_type() else {
            return Ok(());
        };

        Self::check(who, call).map_err(Into::into)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::CheckWeights;
    use crate::{Error, MAX_CRV3_COMMIT_SIZE_BYTES, tests::mock::*};
    use codec::Compact;
    use frame_support::{
        BoundedVec, assert_ok, dispatch::DispatchResultWithPostInfo, traits::ConstU32,
        traits::ExtendedDispatchable,
    };
    use frame_system::Call as SystemCall;
    use pallet_drand::LastStoredRound;
    use sp_core::{H256, U256};
    use sp_runtime::DispatchError;
    use subtensor_runtime_common::{MechId, NetUid};

    fn dispatch_with_ext(call: RuntimeCall, origin: RuntimeOrigin) -> DispatchResultWithPostInfo {
        <CheckWeights<Test> as ExtendedDispatchable<RuntimeCall>>::dispatch_with_extension(
            origin, call,
        )
    }

    fn err(result: DispatchResultWithPostInfo) -> DispatchError {
        result.err().unwrap().error
    }

    fn register_neuron(netuid: NetUid, hotkey: U256, coldkey: U256) {
        add_network(netuid, 1, 0);
        setup_reserves(netuid, DEFAULT_RESERVE.into(), DEFAULT_RESERVE.into());
        register_ok_neuron(netuid, hotkey, coldkey, 0);
    }

    fn set_weights_call(netuid: NetUid, uid: u16) -> RuntimeCall {
        RuntimeCall::SubtensorModule(SubtensorCall::set_weights {
            netuid,
            dests: vec![uid],
            weights: vec![1],
            version_key: 0,
        })
    }

    fn reveal_weights_call(netuid: NetUid) -> RuntimeCall {
        RuntimeCall::SubtensorModule(SubtensorCall::reveal_weights {
            netuid,
            uids: vec![0],
            values: vec![1],
            salt: vec![1],
            version_key: 0,
        })
    }

    fn reveal_mechanism_weights_call(netuid: NetUid, mecid: MechId) -> RuntimeCall {
        RuntimeCall::SubtensorModule(SubtensorCall::reveal_mechanism_weights {
            netuid,
            mecid,
            uids: vec![0],
            values: vec![1],
            salt: vec![1],
            version_key: 0,
        })
    }

    #[test]
    fn unrelated_calls_pass_through() {
        new_test_ext(0).execute_with(|| {
            let call = RuntimeCall::System(SystemCall::remark { remark: vec![] });

            assert_ok!(dispatch_with_ext(
                call,
                RuntimeOrigin::signed(U256::from(1))
            ));
        });
    }

    #[test]
    fn mismatched_batch_lengths_are_blocked() {
        new_test_ext(0).execute_with(|| {
            let netuid = NetUid::from(1);
            let hotkey = U256::from(1);
            let calls = [
                RuntimeCall::SubtensorModule(SubtensorCall::batch_commit_weights {
                    netuids: vec![Compact(netuid)],
                    commit_hashes: vec![],
                }),
                RuntimeCall::SubtensorModule(SubtensorCall::batch_reveal_weights {
                    netuid,
                    uids_list: vec![vec![0]],
                    values_list: vec![],
                    salts_list: vec![vec![1]],
                    version_keys: vec![0],
                }),
                RuntimeCall::SubtensorModule(SubtensorCall::batch_set_weights {
                    netuids: vec![Compact(netuid)],
                    weights: vec![],
                    version_keys: vec![Compact(0_u64)],
                }),
            ];

            for call in calls {
                assert_eq!(
                    err(dispatch_with_ext(call, RuntimeOrigin::signed(hotkey))),
                    Error::<Test>::InputLengthsUnequal.into()
                );
            }
        });
    }

    #[test]
    fn low_stake_weight_calls_are_blocked() {
        new_test_ext(0).execute_with(|| {
            let netuid = NetUid::from(1);
            let hotkey = U256::from(1);
            let coldkey = U256::from(2);
            let bounded_commit =
                BoundedVec::<u8, ConstU32<MAX_CRV3_COMMIT_SIZE_BYTES>>::try_from(vec![0]).unwrap();
            add_network_disable_commit_reveal(netuid, 1, 0);
            setup_reserves(netuid, DEFAULT_RESERVE.into(), DEFAULT_RESERVE.into());
            SubtensorModule::append_neuron(netuid, &hotkey, 0);
            crate::Owner::<Test>::insert(hotkey, coldkey);
            SubtensorModule::set_stake_threshold(1_000_000_000_000_u64);

            let calls = [
                set_weights_call(netuid, 0),
                RuntimeCall::SubtensorModule(SubtensorCall::set_mechanism_weights {
                    netuid,
                    mecid: MechId::MAIN,
                    dests: vec![0],
                    weights: vec![1],
                    version_key: 0,
                }),
                RuntimeCall::SubtensorModule(SubtensorCall::batch_set_weights {
                    netuids: vec![Compact(netuid)],
                    weights: vec![vec![(Compact(0_u16), Compact(1_u16))]],
                    version_keys: vec![Compact(0_u64)],
                }),
                RuntimeCall::SubtensorModule(SubtensorCall::commit_weights {
                    netuid,
                    commit_hash: H256::zero(),
                }),
                RuntimeCall::SubtensorModule(SubtensorCall::commit_mechanism_weights {
                    netuid,
                    mecid: MechId::MAIN,
                    commit_hash: H256::zero(),
                }),
                RuntimeCall::SubtensorModule(SubtensorCall::batch_commit_weights {
                    netuids: vec![Compact(netuid)],
                    commit_hashes: vec![H256::zero()],
                }),
                reveal_weights_call(netuid),
                reveal_mechanism_weights_call(netuid, MechId::MAIN),
                RuntimeCall::SubtensorModule(SubtensorCall::batch_reveal_weights {
                    netuid,
                    uids_list: vec![vec![0]],
                    values_list: vec![vec![1]],
                    salts_list: vec![vec![1]],
                    version_keys: vec![0],
                }),
                RuntimeCall::SubtensorModule(SubtensorCall::commit_timelocked_weights {
                    netuid,
                    commit: bounded_commit.clone(),
                    reveal_round: 0,
                    commit_reveal_version: 0,
                }),
                RuntimeCall::SubtensorModule(SubtensorCall::commit_timelocked_mechanism_weights {
                    netuid,
                    mecid: MechId::MAIN,
                    commit: bounded_commit.clone(),
                    reveal_round: 0,
                    commit_reveal_version: 0,
                }),
                RuntimeCall::SubtensorModule(SubtensorCall::commit_crv3_mechanism_weights {
                    netuid,
                    mecid: MechId::MAIN,
                    commit: bounded_commit,
                    reveal_round: 0,
                }),
            ];

            for call in calls {
                assert_eq!(
                    err(dispatch_with_ext(call, RuntimeOrigin::signed(hotkey))),
                    Error::<Test>::NotEnoughStakeToSetWeights.into()
                );
            }
        });
    }

    #[test]
    fn valid_set_weights_call_dispatches() {
        new_test_ext(0).execute_with(|| {
            let netuid = NetUid::from(1);
            let hotkey = U256::from(1);
            let coldkey = U256::from(2);
            add_network_disable_commit_reveal(netuid, 1, 0);
            setup_reserves(netuid, DEFAULT_RESERVE.into(), DEFAULT_RESERVE.into());
            register_ok_neuron(netuid, hotkey, coldkey, 0);
            SubtensorModule::set_stake_threshold(0);
            let uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey).unwrap();

            assert_ok!(dispatch_with_ext(
                set_weights_call(netuid, uid),
                RuntimeOrigin::signed(hotkey)
            ));
        });
    }

    #[test]
    fn missing_commit_is_blocked() {
        new_test_ext(0).execute_with(|| {
            let netuid = NetUid::from(1);
            let hotkey = U256::from(1);
            let coldkey = U256::from(2);
            register_neuron(netuid, hotkey, coldkey);
            SubtensorModule::set_stake_threshold(0);
            let calls = [
                reveal_weights_call(netuid),
                reveal_mechanism_weights_call(netuid, MechId::MAIN),
                RuntimeCall::SubtensorModule(SubtensorCall::batch_reveal_weights {
                    netuid,
                    uids_list: vec![vec![0]],
                    values_list: vec![vec![1]],
                    salts_list: vec![vec![1]],
                    version_keys: vec![0],
                }),
            ];

            for call in calls {
                assert_eq!(
                    err(dispatch_with_ext(call, RuntimeOrigin::signed(hotkey))),
                    Error::<Test>::NoWeightsCommitFound.into()
                );
            }
        });
    }

    #[test]
    fn reveal_before_window_is_blocked() {
        new_test_ext(0).execute_with(|| {
            let netuid = NetUid::from(1);
            let hotkey = U256::from(1);
            let coldkey = U256::from(2);
            let uids = vec![0];
            let values = vec![1];
            let salt = vec![1];
            let version_key = 0;
            register_neuron(netuid, hotkey, coldkey);
            SubtensorModule::set_commit_reveal_weights_enabled(netuid, true);
            SubtensorModule::set_stake_threshold(0);
            let commit_hash = SubtensorModule::get_commit_hash(
                &hotkey,
                netuid.into(),
                &uids,
                &values,
                &salt,
                version_key,
            );
            assert_ok!(SubtensorModule::commit_weights(
                RuntimeOrigin::signed(hotkey),
                netuid,
                commit_hash,
            ));

            assert_eq!(
                err(dispatch_with_ext(
                    reveal_weights_call(netuid),
                    RuntimeOrigin::signed(hotkey)
                )),
                Error::<Test>::RevealTooEarly.into()
            );
        });
    }

    #[test]
    fn valid_mechanism_reveal_dispatches() {
        for (mecid, mechanism_count) in [
            (MechId::MAIN, None),
            (MechId::from(1_u8), Some(MechId::from(2_u8))),
        ] {
            new_test_ext(0).execute_with(|| {
                let netuid = NetUid::from(1);
                let hotkey = U256::from(1);
                let coldkey = U256::from(2);
                let uids = vec![0];
                let values = vec![1];
                let salt = vec![1];
                let version_key = 0;
                register_neuron(netuid, hotkey, coldkey);
                if let Some(mechanism_count) = mechanism_count {
                    crate::MechanismCountCurrent::<Test>::insert(netuid, mechanism_count);
                }
                SubtensorModule::set_commit_reveal_weights_enabled(netuid, true);
                SubtensorModule::set_stake_threshold(0);

                let commit_hash = SubtensorModule::get_commit_hash(
                    &hotkey,
                    SubtensorModule::get_mechanism_storage_index(netuid, mecid),
                    &uids,
                    &values,
                    &salt,
                    version_key,
                );
                assert_ok!(SubtensorModule::commit_mechanism_weights(
                    RuntimeOrigin::signed(hotkey),
                    netuid,
                    mecid,
                    commit_hash,
                ));
                step_epochs(1, netuid);

                let call = RuntimeCall::SubtensorModule(SubtensorCall::reveal_mechanism_weights {
                    netuid,
                    mecid,
                    uids,
                    values,
                    salt,
                    version_key,
                });
                assert_ok!(dispatch_with_ext(call, RuntimeOrigin::signed(hotkey)));
            });
        }
    }

    #[test]
    fn invalid_reveal_round_is_blocked() {
        new_test_ext(0).execute_with(|| {
            LastStoredRound::<Test>::put(1_000_u64);
            let netuid = NetUid::from(1);
            let hotkey = U256::from(1);
            let coldkey = U256::from(2);
            register_neuron(netuid, hotkey, coldkey);
            SubtensorModule::set_stake_threshold(0);
            let commit =
                BoundedVec::<u8, ConstU32<MAX_CRV3_COMMIT_SIZE_BYTES>>::try_from(vec![0]).unwrap();
            let calls = [
                RuntimeCall::SubtensorModule(SubtensorCall::commit_timelocked_weights {
                    netuid,
                    commit: commit.clone(),
                    reveal_round: 999,
                    commit_reveal_version: 0,
                }),
                RuntimeCall::SubtensorModule(SubtensorCall::commit_timelocked_mechanism_weights {
                    netuid,
                    mecid: MechId::MAIN,
                    commit: commit.clone(),
                    reveal_round: 999,
                    commit_reveal_version: 0,
                }),
                RuntimeCall::SubtensorModule(SubtensorCall::commit_crv3_mechanism_weights {
                    netuid,
                    mecid: MechId::MAIN,
                    commit,
                    reveal_round: 999,
                }),
            ];

            for call in calls {
                assert_eq!(
                    err(dispatch_with_ext(call, RuntimeOrigin::signed(hotkey))),
                    Error::<Test>::InvalidRevealRound.into()
                );
            }
        });
    }
}
