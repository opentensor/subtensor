mod types;

use crate::types::{FunctionId, Output};
use codec::Encode;
use frame_support::{DebugNoBound, traits::Get};
use frame_system::RawOrigin;
use pallet_contracts::chain_extension::{
    ChainExtension, Environment, Ext, InitState, RetVal, SysConfig,
};
use sp_runtime::{DispatchError, Weight, traits::StaticLookup};
use sp_std::marker::PhantomData;
use subtensor_runtime_common::{AlphaCurrency, NetUid, TaoCurrency};

#[derive(DebugNoBound)]
pub struct SubtensorChainExtension<T>(PhantomData<T>);

impl<T> ChainExtension<T> for SubtensorChainExtension<T>
where
    T: pallet_subtensor::Config + pallet_contracts::Config,
    <<T as SysConfig>::Lookup as StaticLookup>::Source: From<<T as SysConfig>::AccountId>,
{
    fn call<E>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
    where
        E: Ext<T = T>,
    {
        let mut env = env.buf_in_buf_out();
        let func_id = env.func_id().try_into().map_err(|_| {
            DispatchError::Other(
                "Invalid function id - does not correspond to any registered function",
            )
        })?;

        match func_id {
            FunctionId::GetStakeInfoForHotkeyColdkeyNetuidV1 => {
                let input: (T::AccountId, T::AccountId, NetUid) = env
                    .read_as()
                    .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;
                let (hotkey, coldkey, netuid) = input;

                let stake_info =
                    pallet_subtensor::Pallet::<T>::get_stake_info_for_hotkey_coldkey_netuid(
                        hotkey, coldkey, netuid,
                    );

                let encoded_result = stake_info.encode();

                env.write(&encoded_result, false, None)
                    .map_err(|_| DispatchError::Other("Failed to write output"))?;

                Ok(RetVal::Converging(0))
            }
            // Weights are copied from the pallet calls. Use `WeightInfo` when available.
            FunctionId::AddStakeV1 => {
                let weight = Weight::from_parts(340_800_000, 0)
                    .saturating_add(T::DbWeight::get().reads(24_u64))
                    .saturating_add(T::DbWeight::get().writes(15));

                env.charge_weight(weight)?;

                let input: (T::AccountId, NetUid, TaoCurrency) = env
                    .read_as()
                    .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;
                let (hotkey, netuid, amount_staked) = input;

                let call_result = pallet_subtensor::Pallet::<T>::add_stake(
                    RawOrigin::Signed(env.ext().address().clone()).into(),
                    hotkey,
                    netuid,
                    amount_staked,
                );

                match call_result {
                    Ok(_) => Ok(RetVal::Converging(Output::Success as u32)),
                    Err(e) => {
                        let error_code = Output::from(e) as u32;
                        Ok(RetVal::Converging(error_code))
                    }
                }
            }
            FunctionId::RemoveStakeV1 => {
                let weight = Weight::from_parts(196_800_000, 0)
                    .saturating_add(T::DbWeight::get().reads(19))
                    .saturating_add(T::DbWeight::get().writes(10));

                env.charge_weight(weight)?;

                let input: (T::AccountId, NetUid, AlphaCurrency) = env
                    .read_as()
                    .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;
                let (hotkey, netuid, amount_unstaked) = input;

                let call_result = pallet_subtensor::Pallet::<T>::remove_stake(
                    RawOrigin::Signed(env.ext().address().clone()).into(),
                    hotkey,
                    netuid,
                    amount_unstaked,
                );

                match call_result {
                    Ok(_) => Ok(RetVal::Converging(Output::Success as u32)),
                    Err(e) => {
                        let error_code = Output::from(e) as u32;
                        Ok(RetVal::Converging(error_code))
                    }
                }
            }
            FunctionId::UnstakeAllV1 => {
                let weight = Weight::from_parts(28_830_000, 0)
                    .saturating_add(T::DbWeight::get().reads(6))
                    .saturating_add(T::DbWeight::get().writes(0));

                env.charge_weight(weight)?;

                let hotkey: T::AccountId = env
                    .read_as()
                    .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;

                let call_result = pallet_subtensor::Pallet::<T>::unstake_all(
                    RawOrigin::Signed(env.ext().address().clone()).into(),
                    hotkey,
                );

                match call_result {
                    Ok(_) => Ok(RetVal::Converging(Output::Success as u32)),
                    Err(e) => {
                        let error_code = Output::from(e) as u32;
                        Ok(RetVal::Converging(error_code))
                    }
                }
            }
            FunctionId::UnstakeAllAlphaV1 => {
                let weight = Weight::from_parts(358_500_000, 0)
                    .saturating_add(T::DbWeight::get().reads(36_u64))
                    .saturating_add(T::DbWeight::get().writes(21_u64));

                env.charge_weight(weight)?;

                let hotkey: T::AccountId = env
                    .read_as()
                    .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;

                let call_result = pallet_subtensor::Pallet::<T>::unstake_all_alpha(
                    RawOrigin::Signed(env.ext().address().clone()).into(),
                    hotkey,
                );

                match call_result {
                    Ok(_) => Ok(RetVal::Converging(Output::Success as u32)),
                    Err(e) => {
                        let error_code = Output::from(e) as u32;
                        Ok(RetVal::Converging(error_code))
                    }
                }
            }
            FunctionId::MoveStakeV1 => {
                let weight = Weight::from_parts(164_300_000, 0)
                    .saturating_add(T::DbWeight::get().reads(15_u64))
                    .saturating_add(T::DbWeight::get().writes(7_u64));

                env.charge_weight(weight)?;

                let input: (T::AccountId, T::AccountId, NetUid, NetUid, AlphaCurrency) = env
                    .read_as()
                    .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;
                let (
                    origin_hotkey,
                    destination_hotkey,
                    origin_netuid,
                    destination_netuid,
                    alpha_amount,
                ) = input;

                let call_result = pallet_subtensor::Pallet::<T>::move_stake(
                    RawOrigin::Signed(env.ext().address().clone()).into(),
                    origin_hotkey,
                    destination_hotkey,
                    origin_netuid,
                    destination_netuid,
                    alpha_amount,
                );

                match call_result {
                    Ok(_) => Ok(RetVal::Converging(Output::Success as u32)),
                    Err(e) => {
                        let error_code = Output::from(e) as u32;
                        Ok(RetVal::Converging(error_code))
                    }
                }
            }
            FunctionId::TransferStakeV1 => {
                let weight = Weight::from_parts(160_300_000, 0)
                    .saturating_add(T::DbWeight::get().reads(13_u64))
                    .saturating_add(T::DbWeight::get().writes(6_u64));

                env.charge_weight(weight)?;

                let input: (T::AccountId, T::AccountId, NetUid, NetUid, AlphaCurrency) = env
                    .read_as()
                    .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;
                let (destination_coldkey, hotkey, origin_netuid, destination_netuid, alpha_amount) =
                    input;

                let call_result = pallet_subtensor::Pallet::<T>::transfer_stake(
                    RawOrigin::Signed(env.ext().address().clone()).into(),
                    destination_coldkey,
                    hotkey,
                    origin_netuid,
                    destination_netuid,
                    alpha_amount,
                );

                match call_result {
                    Ok(_) => Ok(RetVal::Converging(Output::Success as u32)),
                    Err(e) => {
                        let error_code = Output::from(e) as u32;
                        Ok(RetVal::Converging(error_code))
                    }
                }
            }
            FunctionId::SwapStakeV1 => {
                let weight = Weight::from_parts(351_300_000, 0)
                    .saturating_add(T::DbWeight::get().reads(35_u64))
                    .saturating_add(T::DbWeight::get().writes(22_u64));

                env.charge_weight(weight)?;

                let input: (T::AccountId, NetUid, NetUid, AlphaCurrency) = env
                    .read_as()
                    .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;
                let (hotkey, origin_netuid, destination_netuid, alpha_amount) = input;

                let call_result = pallet_subtensor::Pallet::<T>::swap_stake(
                    RawOrigin::Signed(env.ext().address().clone()).into(),
                    hotkey,
                    origin_netuid,
                    destination_netuid,
                    alpha_amount,
                );

                match call_result {
                    Ok(_) => Ok(RetVal::Converging(Output::Success as u32)),
                    Err(e) => {
                        let error_code = Output::from(e) as u32;
                        Ok(RetVal::Converging(error_code))
                    }
                }
            }
            FunctionId::AddStakeLimitV1 => {
                let weight = Weight::from_parts(402_900_000, 0)
                    .saturating_add(T::DbWeight::get().reads(24_u64))
                    .saturating_add(T::DbWeight::get().writes(15));

                env.charge_weight(weight)?;

                let input: (T::AccountId, NetUid, TaoCurrency, TaoCurrency, bool) =
                    env.read_as()
                        .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;
                let (hotkey, netuid, amount_staked, limit_price, allow_partial) = input;

                let call_result = pallet_subtensor::Pallet::<T>::add_stake_limit(
                    RawOrigin::Signed(env.ext().address().clone()).into(),
                    hotkey,
                    netuid,
                    amount_staked,
                    limit_price,
                    allow_partial,
                );

                match call_result {
                    Ok(_) => Ok(RetVal::Converging(Output::Success as u32)),
                    Err(e) => {
                        let error_code = Output::from(e) as u32;
                        Ok(RetVal::Converging(error_code))
                    }
                }
            }
            FunctionId::RemoveStakeLimitV1 => {
                let weight = Weight::from_parts(377_400_000, 0)
                    .saturating_add(T::DbWeight::get().reads(28_u64))
                    .saturating_add(T::DbWeight::get().writes(14));

                env.charge_weight(weight)?;

                let input: (T::AccountId, NetUid, AlphaCurrency, TaoCurrency, bool) = env
                    .read_as()
                    .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;
                let (hotkey, netuid, amount_unstaked, limit_price, allow_partial) = input;

                let call_result = pallet_subtensor::Pallet::<T>::remove_stake_limit(
                    RawOrigin::Signed(env.ext().address().clone()).into(),
                    hotkey,
                    netuid,
                    amount_unstaked,
                    limit_price,
                    allow_partial,
                );

                match call_result {
                    Ok(_) => Ok(RetVal::Converging(Output::Success as u32)),
                    Err(e) => {
                        let error_code = Output::from(e) as u32;
                        Ok(RetVal::Converging(error_code))
                    }
                }
            }
            FunctionId::SwapStakeLimitV1 => {
                let weight = Weight::from_parts(411_500_000, 0)
                    .saturating_add(T::DbWeight::get().reads(35_u64))
                    .saturating_add(T::DbWeight::get().writes(22_u64));

                env.charge_weight(weight)?;

                let input: (
                    T::AccountId,
                    NetUid,
                    NetUid,
                    AlphaCurrency,
                    TaoCurrency,
                    bool,
                ) = env
                    .read_as()
                    .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;
                let (
                    hotkey,
                    origin_netuid,
                    destination_netuid,
                    alpha_amount,
                    limit_price,
                    allow_partial,
                ) = input;

                let call_result = pallet_subtensor::Pallet::<T>::swap_stake_limit(
                    RawOrigin::Signed(env.ext().address().clone()).into(),
                    hotkey,
                    origin_netuid,
                    destination_netuid,
                    alpha_amount,
                    limit_price,
                    allow_partial,
                );

                match call_result {
                    Ok(_) => Ok(RetVal::Converging(Output::Success as u32)),
                    Err(e) => {
                        let error_code = Output::from(e) as u32;
                        Ok(RetVal::Converging(error_code))
                    }
                }
            }
            FunctionId::RemoveStakeFullLimitV1 => {
                let weight = Weight::from_parts(395_300_000, 0)
                    .saturating_add(T::DbWeight::get().reads(28_u64))
                    .saturating_add(T::DbWeight::get().writes(14_u64));

                env.charge_weight(weight)?;

                let input: (T::AccountId, NetUid, Option<TaoCurrency>) = env
                    .read_as()
                    .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;
                let (hotkey, netuid, limit_price) = input;

                let call_result = pallet_subtensor::Pallet::<T>::remove_stake_full_limit(
                    RawOrigin::Signed(env.ext().address().clone()).into(),
                    hotkey,
                    netuid,
                    limit_price,
                );

                match call_result {
                    Ok(_) => Ok(RetVal::Converging(Output::Success as u32)),
                    Err(e) => {
                        let error_code = Output::from(e) as u32;
                        Ok(RetVal::Converging(error_code))
                    }
                }
            }
            FunctionId::SetColdkeyAutoStakeHotkeyV1 => {
                let weight = Weight::from_parts(29_930_000, 0)
                    .saturating_add(T::DbWeight::get().reads(4_u64))
                    .saturating_add(T::DbWeight::get().writes(2_u64));

                env.charge_weight(weight)?;

                let input: (NetUid, T::AccountId) = env
                    .read_as()
                    .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;
                let (netuid, hotkey) = input;

                let call_result = pallet_subtensor::Pallet::<T>::set_coldkey_auto_stake_hotkey(
                    RawOrigin::Signed(env.ext().address().clone()).into(),
                    netuid,
                    hotkey,
                );

                match call_result {
                    Ok(_) => Ok(RetVal::Converging(Output::Success as u32)),
                    Err(e) => {
                        let error_code = Output::from(e) as u32;
                        Ok(RetVal::Converging(error_code))
                    }
                }
            }
        }
    }

    fn enabled() -> bool {
        true
    }
}
