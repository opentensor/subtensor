#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod types;

use crate::types::{FunctionId, Output};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    DebugNoBound,
    dispatch::{DispatchInfo, GetDispatchInfo, PostDispatchInfo},
    traits::Get,
};
use frame_system::RawOrigin;
use pallet_contracts::chain_extension::{
    BufInBufOutState, ChainExtension, Environment, Ext, InitState, RetVal, SysConfig,
};
use pallet_subtensor::weights::WeightInfo as SubtensorWeightInfo;
use pallet_subtensor_proxy as pallet_proxy;
use pallet_subtensor_proxy::WeightInfo;
use sp_runtime::{
    DispatchError, Weight,
    traits::{AsSystemOriginSigner, Dispatchable, StaticLookup},
    transaction_validity::{InvalidTransaction, TransactionValidityError},
};
use sp_std::marker::PhantomData;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{
    AlphaBalance, NetUid, ProxyType, RuntimeTxExtensionProvider, TaoBalance, TxExtDispatchError,
    dispatch_with_tx_extensions,
};
use subtensor_swap_interface::SwapHandler;

#[derive(DebugNoBound)]
pub struct SubtensorChainExtension<T>(PhantomData<T>);

impl<T> Default for SubtensorChainExtension<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T> ChainExtension<T> for SubtensorChainExtension<T>
where
    T: pallet_subtensor::Config
        + pallet_contracts::Config
        + pallet_proxy::Config<ProxyType = ProxyType>
        + pallet_subtensor_swap::Config
        + RuntimeTxExtensionProvider
        + Send
        + Sync,
    T::AccountId: Clone,
    <T as SysConfig>::RuntimeCall: From<pallet_subtensor::Call<T>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <T as SysConfig>::RuntimeOrigin:
        From<RawOrigin<T::AccountId>> + AsSystemOriginSigner<T::AccountId> + Clone,
    <<T as SysConfig>::Lookup as StaticLookup>::Source: From<<T as SysConfig>::AccountId>,
{
    fn call<E>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
    where
        E: Ext<T = T>,
    {
        let mut adapter = ContractsEnvAdapter::<T, E>::new(env);
        Self::dispatch(&mut adapter)
    }

    fn enabled() -> bool {
        true
    }
}

impl<T> SubtensorChainExtension<T>
where
    T: pallet_subtensor::Config
        + pallet_contracts::Config
        + pallet_proxy::Config<ProxyType = ProxyType>
        + pallet_subtensor_swap::Config
        + RuntimeTxExtensionProvider
        + Send
        + Sync,
    T::AccountId: Clone,
    <T as SysConfig>::RuntimeCall: From<pallet_subtensor::Call<T>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <T as SysConfig>::RuntimeOrigin:
        From<RawOrigin<T::AccountId>> + AsSystemOriginSigner<T::AccountId> + Clone,
{
    /// Dispatch a `pallet-subtensor` call through the runtime transaction-extension tuple, so the
    /// chain-extension path runs the same checks (notably rate limiting) and records the same usage
    /// as a normal extrinsic, then map the outcome to a chain-extension [`Output`] code.
    ///
    /// Calling the pallet dispatchables directly (as plain Rust functions) would skip the extension
    /// tuple entirely and bypass rate limiting; routing through [`dispatch_with_tx_extensions`]
    /// closes that gap and keeps this path consistent with the EVM precompiles.
    fn dispatch_subtensor_call(
        origin: RawOrigin<T::AccountId>,
        call: pallet_subtensor::Call<T>,
    ) -> Result<RetVal, DispatchError> {
        let code = match dispatch_with_tx_extensions::<T, _>(call, origin) {
            Ok(_) => Output::Success as u32,
            Err(TxExtDispatchError::Dispatch(e)) => Output::from(e) as u32,
            Err(TxExtDispatchError::Extension(e)) => match e {
                TransactionValidityError::Invalid(InvalidTransaction::Custom(code))
                    if code == pallet_rate_limiting::RATE_LIMIT_DENIED =>
                {
                    Output::TxRateLimitExceeded as u32
                }
                _ => Output::RuntimeError as u32,
            },
        };

        Ok(RetVal::Converging(code))
    }

    fn dispatch_add_stake_v1<Env>(
        env: &mut Env,
        origin: RawOrigin<T::AccountId>,
    ) -> Result<RetVal, DispatchError>
    where
        Env: SubtensorExtensionEnv<T>,
        <<T as SysConfig>::Lookup as StaticLookup>::Source: From<<T as SysConfig>::AccountId>,
    {
        let (hotkey, netuid, amount_staked): (T::AccountId, NetUid, TaoBalance) = env
            .read_as()
            .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;

        let weight =
            <<T as pallet_subtensor::Config>::WeightInfo as SubtensorWeightInfo>::add_stake();

        env.charge_weight(weight)?;

        Self::dispatch_subtensor_call(
            origin,
            pallet_subtensor::Call::<T>::add_stake {
                hotkey,
                netuid,
                amount_staked,
            },
        )
    }

    fn dispatch_remove_stake_v1<Env>(
        env: &mut Env,
        origin: RawOrigin<T::AccountId>,
    ) -> Result<RetVal, DispatchError>
    where
        Env: SubtensorExtensionEnv<T>,
        <<T as SysConfig>::Lookup as StaticLookup>::Source: From<<T as SysConfig>::AccountId>,
    {
        let (hotkey, netuid, amount_unstaked): (T::AccountId, NetUid, AlphaBalance) = env
            .read_as()
            .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;

        // weight for remove_stake is not defined in the Subtensor pallet's WeightInfo
        let weight =
            <<T as pallet_subtensor::Config>::WeightInfo as SubtensorWeightInfo>::remove_stake();

        env.charge_weight(weight)?;

        Self::dispatch_subtensor_call(
            origin,
            pallet_subtensor::Call::<T>::remove_stake {
                hotkey,
                netuid,
                amount_unstaked,
            },
        )
    }

    fn dispatch_unstake_all_v1<Env>(
        env: &mut Env,
        origin: RawOrigin<T::AccountId>,
    ) -> Result<RetVal, DispatchError>
    where
        Env: SubtensorExtensionEnv<T>,
        <<T as SysConfig>::Lookup as StaticLookup>::Source: From<<T as SysConfig>::AccountId>,
    {
        let hotkey: T::AccountId = env
            .read_as()
            .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;

        let weight =
            <<T as pallet_subtensor::Config>::WeightInfo as SubtensorWeightInfo>::unstake_all();

        env.charge_weight(weight)?;

        Self::dispatch_subtensor_call(origin, pallet_subtensor::Call::<T>::unstake_all { hotkey })
    }

    fn dispatch_unstake_all_alpha_v1<Env>(
        env: &mut Env,
        origin: RawOrigin<T::AccountId>,
    ) -> Result<RetVal, DispatchError>
    where
        Env: SubtensorExtensionEnv<T>,
        <<T as SysConfig>::Lookup as StaticLookup>::Source: From<<T as SysConfig>::AccountId>,
    {
        let hotkey: T::AccountId = env
            .read_as()
            .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;

        let weight =
            <<T as pallet_subtensor::Config>::WeightInfo as SubtensorWeightInfo>::unstake_all_alpha(
            );

        env.charge_weight(weight)?;

        Self::dispatch_subtensor_call(
            origin,
            pallet_subtensor::Call::<T>::unstake_all_alpha { hotkey },
        )
    }

    fn dispatch_move_stake_v1<Env>(
        env: &mut Env,
        origin: RawOrigin<T::AccountId>,
    ) -> Result<RetVal, DispatchError>
    where
        Env: SubtensorExtensionEnv<T>,
        <<T as SysConfig>::Lookup as StaticLookup>::Source: From<<T as SysConfig>::AccountId>,
    {
        let (origin_hotkey, destination_hotkey, origin_netuid, destination_netuid, alpha_amount): (
            T::AccountId,
            T::AccountId,
            NetUid,
            NetUid,
            AlphaBalance,
        ) = env
            .read_as()
            .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;

        let weight =
            <<T as pallet_subtensor::Config>::WeightInfo as SubtensorWeightInfo>::move_stake();

        env.charge_weight(weight)?;

        Self::dispatch_subtensor_call(
            origin,
            pallet_subtensor::Call::<T>::move_stake {
                origin_hotkey,
                destination_hotkey,
                origin_netuid,
                destination_netuid,
                alpha_amount,
            },
        )
    }

    fn dispatch_transfer_stake_v1<Env>(
        env: &mut Env,
        origin: RawOrigin<T::AccountId>,
    ) -> Result<RetVal, DispatchError>
    where
        Env: SubtensorExtensionEnv<T>,
        <<T as SysConfig>::Lookup as StaticLookup>::Source: From<<T as SysConfig>::AccountId>,
    {
        let (destination_coldkey, hotkey, origin_netuid, destination_netuid, alpha_amount): (
            T::AccountId,
            T::AccountId,
            NetUid,
            NetUid,
            AlphaBalance,
        ) = env
            .read_as()
            .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;

        let weight =
            <<T as pallet_subtensor::Config>::WeightInfo as SubtensorWeightInfo>::transfer_stake();

        env.charge_weight(weight)?;

        Self::dispatch_subtensor_call(
            origin,
            pallet_subtensor::Call::<T>::transfer_stake {
                destination_coldkey,
                hotkey,
                origin_netuid,
                destination_netuid,
                alpha_amount,
            },
        )
    }

    fn dispatch_swap_stake_v1<Env>(
        env: &mut Env,
        origin: RawOrigin<T::AccountId>,
    ) -> Result<RetVal, DispatchError>
    where
        Env: SubtensorExtensionEnv<T>,
        <<T as SysConfig>::Lookup as StaticLookup>::Source: From<<T as SysConfig>::AccountId>,
    {
        let (hotkey, origin_netuid, destination_netuid, alpha_amount): (
            T::AccountId,
            NetUid,
            NetUid,
            AlphaBalance,
        ) = env
            .read_as()
            .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;

        let weight =
            <<T as pallet_subtensor::Config>::WeightInfo as SubtensorWeightInfo>::swap_stake();

        env.charge_weight(weight)?;

        Self::dispatch_subtensor_call(
            origin,
            pallet_subtensor::Call::<T>::swap_stake {
                hotkey,
                origin_netuid,
                destination_netuid,
                alpha_amount,
            },
        )
    }

    fn dispatch_add_stake_limit_v1<Env>(
        env: &mut Env,
        origin: RawOrigin<T::AccountId>,
    ) -> Result<RetVal, DispatchError>
    where
        Env: SubtensorExtensionEnv<T>,
        <<T as SysConfig>::Lookup as StaticLookup>::Source: From<<T as SysConfig>::AccountId>,
    {
        let (hotkey, netuid, amount_staked, limit_price, allow_partial): (
            T::AccountId,
            NetUid,
            TaoBalance,
            TaoBalance,
            bool,
        ) = env
            .read_as()
            .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;

        let weight =
            <<T as pallet_subtensor::Config>::WeightInfo as SubtensorWeightInfo>::add_stake_limit();

        env.charge_weight(weight)?;

        Self::dispatch_subtensor_call(
            origin,
            pallet_subtensor::Call::<T>::add_stake_limit {
                hotkey,
                netuid,
                amount_staked,
                limit_price,
                allow_partial,
            },
        )
    }

    fn dispatch_remove_stake_limit_v1<Env>(
        env: &mut Env,
        origin: RawOrigin<T::AccountId>,
    ) -> Result<RetVal, DispatchError>
    where
        Env: SubtensorExtensionEnv<T>,
        <<T as SysConfig>::Lookup as StaticLookup>::Source: From<<T as SysConfig>::AccountId>,
    {
        let (hotkey, netuid, amount_unstaked, limit_price, allow_partial): (
            T::AccountId,
            NetUid,
            AlphaBalance,
            TaoBalance,
            bool,
        ) = env
            .read_as()
            .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;

        let weight =
            <<T as pallet_subtensor::Config>::WeightInfo as SubtensorWeightInfo>::remove_stake_limit();

        env.charge_weight(weight)?;

        Self::dispatch_subtensor_call(
            origin,
            pallet_subtensor::Call::<T>::remove_stake_limit {
                hotkey,
                netuid,
                amount_unstaked,
                limit_price,
                allow_partial,
            },
        )
    }

    fn dispatch_swap_stake_limit_v1<Env>(
        env: &mut Env,
        origin: RawOrigin<T::AccountId>,
    ) -> Result<RetVal, DispatchError>
    where
        Env: SubtensorExtensionEnv<T>,
        <<T as SysConfig>::Lookup as StaticLookup>::Source: From<<T as SysConfig>::AccountId>,
    {
        let (
            hotkey,
            origin_netuid,
            destination_netuid,
            alpha_amount,
            limit_price,
            allow_partial,
        ): (T::AccountId, NetUid, NetUid, AlphaBalance, TaoBalance, bool) =
            env.read_as()
                .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;

        let weight =
            <<T as pallet_subtensor::Config>::WeightInfo as SubtensorWeightInfo>::swap_stake_limit(
            );

        env.charge_weight(weight)?;

        Self::dispatch_subtensor_call(
            origin,
            pallet_subtensor::Call::<T>::swap_stake_limit {
                hotkey,
                origin_netuid,
                destination_netuid,
                alpha_amount,
                limit_price,
                allow_partial,
            },
        )
    }

    fn dispatch_remove_stake_full_limit_v1<Env>(
        env: &mut Env,
        origin: RawOrigin<T::AccountId>,
    ) -> Result<RetVal, DispatchError>
    where
        Env: SubtensorExtensionEnv<T>,
        <<T as SysConfig>::Lookup as StaticLookup>::Source: From<<T as SysConfig>::AccountId>,
    {
        let (hotkey, netuid, limit_price): (T::AccountId, NetUid, Option<TaoBalance>) = env
            .read_as()
            .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;

        let weight = <<T as pallet_subtensor::Config>::WeightInfo as SubtensorWeightInfo>::remove_stake_full_limit();

        env.charge_weight(weight)?;

        Self::dispatch_subtensor_call(
            origin,
            pallet_subtensor::Call::<T>::remove_stake_full_limit {
                hotkey,
                netuid,
                limit_price,
            },
        )
    }

    fn dispatch_set_coldkey_auto_stake_hotkey_v1<Env>(
        env: &mut Env,
        origin: RawOrigin<T::AccountId>,
    ) -> Result<RetVal, DispatchError>
    where
        Env: SubtensorExtensionEnv<T>,
        <<T as SysConfig>::Lookup as StaticLookup>::Source: From<<T as SysConfig>::AccountId>,
    {
        let (netuid, hotkey): (NetUid, T::AccountId) = env
            .read_as()
            .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;

        let weight = <<T as pallet_subtensor::Config>::WeightInfo as SubtensorWeightInfo>::set_coldkey_auto_stake_hotkey();

        env.charge_weight(weight)?;

        Self::dispatch_subtensor_call(
            origin,
            pallet_subtensor::Call::<T>::set_coldkey_auto_stake_hotkey { netuid, hotkey },
        )
    }

    fn dispatch_add_proxy_v1<Env>(
        env: &mut Env,
        origin: RawOrigin<T::AccountId>,
    ) -> Result<RetVal, DispatchError>
    where
        Env: SubtensorExtensionEnv<T>,
        <<T as SysConfig>::Lookup as StaticLookup>::Source: From<<T as SysConfig>::AccountId>,
    {
        let delegate: T::AccountId = env
            .read_as()
            .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;

        let weight = <T as pallet_proxy::Config>::WeightInfo::add_proxy(
            <T as pallet_proxy::Config>::MaxProxies::get(),
        );

        env.charge_weight(weight)?;

        let delegate_lookup =
            <<T as frame_system::Config>::Lookup as StaticLookup>::Source::from(delegate);

        let call_result = pallet_proxy::Pallet::<T>::add_proxy(
            origin.into(),
            delegate_lookup,
            ProxyType::Staking,
            0u32.into(),
        );

        match call_result {
            Ok(_) => Ok(RetVal::Converging(Output::Success as u32)),
            Err(e) => {
                let error_code = Output::from(e) as u32;
                Ok(RetVal::Converging(error_code))
            }
        }
    }

    fn dispatch_remove_proxy_v1<Env>(
        env: &mut Env,
        origin: RawOrigin<T::AccountId>,
    ) -> Result<RetVal, DispatchError>
    where
        Env: SubtensorExtensionEnv<T>,
        <<T as SysConfig>::Lookup as StaticLookup>::Source: From<<T as SysConfig>::AccountId>,
    {
        let delegate: T::AccountId = env
            .read_as()
            .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;

        let weight = <T as pallet_proxy::Config>::WeightInfo::remove_proxy(
            <T as pallet_proxy::Config>::MaxProxies::get(),
        );

        env.charge_weight(weight)?;

        let delegate_lookup =
            <<T as frame_system::Config>::Lookup as StaticLookup>::Source::from(delegate);

        let call_result = pallet_proxy::Pallet::<T>::remove_proxy(
            origin.into(),
            delegate_lookup,
            ProxyType::Staking,
            0u32.into(),
        );

        match call_result {
            Ok(_) => Ok(RetVal::Converging(Output::Success as u32)),
            Err(e) => {
                let error_code = Output::from(e) as u32;
                Ok(RetVal::Converging(error_code))
            }
        }
    }

    fn dispatch<Env>(env: &mut Env) -> Result<RetVal, DispatchError>
    where
        Env: SubtensorExtensionEnv<T>,
        <<T as SysConfig>::Lookup as StaticLookup>::Source: From<<T as SysConfig>::AccountId>,
    {
        let func_id: FunctionId = env.func_id().try_into().map_err(|_| {
            DispatchError::Other(
                "Invalid function id - does not correspond to any registered function",
            )
        })?;

        match func_id {
            FunctionId::GetStakeInfoForHotkeyColdkeyNetuidV1 => {
                let (hotkey, coldkey, netuid): (T::AccountId, T::AccountId, NetUid) = env
                    .read_as()
                    .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;

                let stake_info =
                    pallet_subtensor::Pallet::<T>::get_stake_info_for_hotkey_coldkey_netuid(
                        hotkey, coldkey, netuid,
                    );

                let encoded_result = stake_info.encode();

                env.write_output(&encoded_result)
                    .map_err(|_| DispatchError::Other("Failed to write output"))?;

                Ok(RetVal::Converging(Output::Success as u32))
            }
            FunctionId::AddStakeV1 => {
                let origin = RawOrigin::Signed(env.caller());
                Self::dispatch_add_stake_v1(env, origin)
            }

            FunctionId::CallerAddStakeV1 => {
                let origin = convert_origin(env.origin());
                Self::dispatch_add_stake_v1(env, origin)
            }

            FunctionId::RemoveStakeV1 => {
                let origin = RawOrigin::Signed(env.caller());
                Self::dispatch_remove_stake_v1(env, origin)
            }
            FunctionId::CallerRemoveStakeV1 => {
                let origin = convert_origin(env.origin());
                Self::dispatch_remove_stake_v1(env, origin)
            }
            FunctionId::UnstakeAllV1 => {
                let origin = RawOrigin::Signed(env.caller());
                Self::dispatch_unstake_all_v1(env, origin)
            }
            FunctionId::CallerUnstakeAllV1 => {
                let origin = convert_origin(env.origin());
                Self::dispatch_unstake_all_v1(env, origin)
            }
            FunctionId::UnstakeAllAlphaV1 => {
                let origin = RawOrigin::Signed(env.caller());
                Self::dispatch_unstake_all_alpha_v1(env, origin)
            }
            FunctionId::CallerUnstakeAllAlphaV1 => {
                let origin = convert_origin(env.origin());
                Self::dispatch_unstake_all_alpha_v1(env, origin)
            }
            FunctionId::MoveStakeV1 => {
                let origin = RawOrigin::Signed(env.caller());
                Self::dispatch_move_stake_v1(env, origin)
            }
            FunctionId::CallerMoveStakeV1 => {
                let origin = convert_origin(env.origin());
                Self::dispatch_move_stake_v1(env, origin)
            }
            FunctionId::TransferStakeV1 => {
                let origin = RawOrigin::Signed(env.caller());
                Self::dispatch_transfer_stake_v1(env, origin)
            }
            FunctionId::CallerTransferStakeV1 => {
                let origin = convert_origin(env.origin());
                Self::dispatch_transfer_stake_v1(env, origin)
            }
            FunctionId::SwapStakeV1 => {
                let origin = RawOrigin::Signed(env.caller());
                Self::dispatch_swap_stake_v1(env, origin)
            }
            FunctionId::CallerSwapStakeV1 => {
                let origin = convert_origin(env.origin());
                Self::dispatch_swap_stake_v1(env, origin)
            }
            FunctionId::AddStakeLimitV1 => {
                let origin = RawOrigin::Signed(env.caller());
                Self::dispatch_add_stake_limit_v1(env, origin)
            }
            FunctionId::CallerAddStakeLimitV1 => {
                let origin = convert_origin(env.origin());
                Self::dispatch_add_stake_limit_v1(env, origin)
            }
            FunctionId::RemoveStakeLimitV1 => {
                let origin = RawOrigin::Signed(env.caller());
                Self::dispatch_remove_stake_limit_v1(env, origin)
            }
            FunctionId::CallerRemoveStakeLimitV1 => {
                let origin = convert_origin(env.origin());
                Self::dispatch_remove_stake_limit_v1(env, origin)
            }
            FunctionId::SwapStakeLimitV1 => {
                let origin = RawOrigin::Signed(env.caller());
                Self::dispatch_swap_stake_limit_v1(env, origin)
            }
            FunctionId::CallerSwapStakeLimitV1 => {
                let origin = convert_origin(env.origin());
                Self::dispatch_swap_stake_limit_v1(env, origin)
            }
            FunctionId::RemoveStakeFullLimitV1 => {
                let origin = RawOrigin::Signed(env.caller());
                Self::dispatch_remove_stake_full_limit_v1(env, origin)
            }
            FunctionId::CallerRemoveStakeFullLimitV1 => {
                let origin = convert_origin(env.origin());
                Self::dispatch_remove_stake_full_limit_v1(env, origin)
            }
            FunctionId::SetColdkeyAutoStakeHotkeyV1 => {
                let origin = RawOrigin::Signed(env.caller());
                Self::dispatch_set_coldkey_auto_stake_hotkey_v1(env, origin)
            }
            FunctionId::CallerSetColdkeyAutoStakeHotkeyV1 => {
                let origin = convert_origin(env.origin());
                Self::dispatch_set_coldkey_auto_stake_hotkey_v1(env, origin)
            }
            FunctionId::AddProxyV1 => {
                let origin = RawOrigin::Signed(env.caller());
                Self::dispatch_add_proxy_v1(env, origin)
            }
            FunctionId::CallerAddProxyV1 => {
                let origin = convert_origin(env.origin());
                Self::dispatch_add_proxy_v1(env, origin)
            }
            FunctionId::RemoveProxyV1 => {
                let origin = RawOrigin::Signed(env.caller());
                Self::dispatch_remove_proxy_v1(env, origin)
            }
            FunctionId::CallerRemoveProxyV1 => {
                let origin = convert_origin(env.origin());
                Self::dispatch_remove_proxy_v1(env, origin)
            }
            FunctionId::GetAlphaPriceV1 => {
                let netuid: NetUid = env
                    .read_as()
                    .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;

                let current_alpha_price =
                    <pallet_subtensor_swap::Pallet<T> as SwapHandler>::current_alpha_price(
                        netuid.into(),
                    );

                let price = current_alpha_price.saturating_mul(U96F32::from_num(1_000_000_000));
                let price: u64 = price.saturating_to_num();

                let encoded_result = price.encode();

                env.write_output(&encoded_result)
                    .map_err(|_| DispatchError::Other("Failed to write output"))?;

                Ok(RetVal::Converging(Output::Success as u32))
            }
            FunctionId::RecycleAlphaV1 => {
                let weight =
                    <<T as pallet_subtensor::Config>::WeightInfo as SubtensorWeightInfo>::recycle_alpha();

                env.charge_weight(weight)?;

                let (hotkey, netuid, amount): (T::AccountId, NetUid, AlphaBalance) =
                    env.read_as()?;

                let caller = env.caller();

                let call_result = pallet_subtensor::Pallet::<T>::do_recycle_alpha(
                    RawOrigin::Signed(caller).into(),
                    hotkey,
                    amount,
                    netuid,
                );

                match call_result {
                    Ok(real_amount) => {
                        env.write_output(&real_amount.encode())
                            .map_err(|_| DispatchError::Other("Failed to write output"))?;
                        Ok(RetVal::Converging(Output::Success as u32))
                    }
                    Err(e) => {
                        let error_code = Output::from(e) as u32;
                        Ok(RetVal::Converging(error_code))
                    }
                }
            }
            FunctionId::BurnAlphaV1 => {
                let weight =
                    <<T as pallet_subtensor::Config>::WeightInfo as SubtensorWeightInfo>::burn_alpha();

                env.charge_weight(weight)?;

                let (hotkey, netuid, amount): (T::AccountId, NetUid, AlphaBalance) =
                    env.read_as()?;

                let caller = env.caller();

                let call_result = pallet_subtensor::Pallet::<T>::do_burn_alpha(
                    RawOrigin::Signed(caller).into(),
                    hotkey,
                    amount,
                    netuid,
                );

                match call_result {
                    Ok(real_amount) => {
                        env.write_output(&real_amount.encode())
                            .map_err(|_| DispatchError::Other("Failed to write output"))?;
                        Ok(RetVal::Converging(Output::Success as u32))
                    }
                    Err(e) => {
                        let error_code = Output::from(e) as u32;
                        Ok(RetVal::Converging(error_code))
                    }
                }
            }
            FunctionId::AddStakeRecycleV1 => {
                let weight =
                    <<T as pallet_subtensor::Config>::WeightInfo as SubtensorWeightInfo>::add_stake()
                        .saturating_add(
                            <<T as pallet_subtensor::Config>::WeightInfo as SubtensorWeightInfo>::recycle_alpha(),
                        );

                env.charge_weight(weight)?;

                let (hotkey, netuid, tao_amount): (T::AccountId, NetUid, TaoBalance) =
                    env.read_as()?;

                let call_result = pallet_subtensor::Pallet::<T>::do_add_stake_recycle(
                    RawOrigin::Signed(env.caller()).into(),
                    hotkey,
                    netuid,
                    tao_amount,
                );

                match call_result {
                    Ok(alpha) => {
                        env.write_output(&alpha.encode())
                            .map_err(|_| DispatchError::Other("Failed to write output"))?;
                        Ok(RetVal::Converging(Output::Success as u32))
                    }
                    Err(e) => {
                        let error_code = Output::from(e) as u32;
                        Ok(RetVal::Converging(error_code))
                    }
                }
            }
            FunctionId::AddStakeBurnV1 => {
                let weight =
                    <<T as pallet_subtensor::Config>::WeightInfo as SubtensorWeightInfo>::add_stake()
                        .saturating_add(
                            <<T as pallet_subtensor::Config>::WeightInfo as SubtensorWeightInfo>::burn_alpha(),
                        );

                env.charge_weight(weight)?;

                let (hotkey, netuid, tao_amount): (T::AccountId, NetUid, TaoBalance) =
                    env.read_as()?;

                let call_result = pallet_subtensor::Pallet::<T>::do_add_stake_burn_permissionless(
                    RawOrigin::Signed(env.caller()).into(),
                    hotkey,
                    netuid,
                    tao_amount,
                );

                match call_result {
                    Ok(alpha) => {
                        env.write_output(&alpha.encode())
                            .map_err(|_| DispatchError::Other("Failed to write output"))?;
                        Ok(RetVal::Converging(Output::Success as u32))
                    }
                    Err(e) => {
                        let error_code = Output::from(e) as u32;
                        Ok(RetVal::Converging(error_code))
                    }
                }
            }
        }
    }
}

// Convert from the contract origin to the raw origin
fn convert_origin<T>(origin: pallet_contracts::Origin<T>) -> RawOrigin<T::AccountId>
where
    T: pallet_contracts::Config,
{
    match origin {
        pallet_contracts::Origin::Signed(caller) => RawOrigin::Signed(caller),
        pallet_contracts::Origin::Root => RawOrigin::Root,
    }
}

trait SubtensorExtensionEnv<T>
where
    T: pallet_contracts::Config,
{
    fn func_id(&self) -> u16;
    fn charge_weight(&mut self, weight: Weight) -> Result<(), DispatchError>;
    fn read_as<U: Decode + MaxEncodedLen>(&mut self) -> Result<U, DispatchError>;
    fn write_output(&mut self, data: &[u8]) -> Result<(), DispatchError>;
    fn caller(&mut self) -> T::AccountId;
    #[allow(dead_code)]
    fn origin(&mut self) -> pallet_contracts::Origin<T>;
}

struct ContractsEnvAdapter<'a, 'b, T, E>
where
    T: pallet_subtensor::Config + pallet_contracts::Config,
    E: Ext<T = T>,
{
    env: Environment<'a, 'b, E, BufInBufOutState>,
    _marker: PhantomData<T>,
}

impl<'a, 'b, T, E> ContractsEnvAdapter<'a, 'b, T, E>
where
    T: pallet_subtensor::Config + pallet_contracts::Config,
    T::AccountId: Clone,
    E: Ext<T = T>,
{
    fn new(env: Environment<'a, 'b, E, InitState>) -> Self {
        Self {
            env: env.buf_in_buf_out(),
            _marker: PhantomData,
        }
    }
}

impl<'a, 'b, T, E> SubtensorExtensionEnv<T> for ContractsEnvAdapter<'a, 'b, T, E>
where
    T: pallet_subtensor::Config + pallet_contracts::Config,
    T::AccountId: Clone,
    E: Ext<T = T>,
{
    fn func_id(&self) -> u16 {
        self.env.func_id()
    }

    fn charge_weight(&mut self, weight: Weight) -> Result<(), DispatchError> {
        self.env.charge_weight(weight).map(|_| ())
    }

    fn read_as<U: Decode + MaxEncodedLen>(&mut self) -> Result<U, DispatchError> {
        self.env.read_as()
    }

    fn write_output(&mut self, data: &[u8]) -> Result<(), DispatchError> {
        self.env.write(data, false, None)
    }

    fn caller(&mut self) -> T::AccountId {
        self.env.ext().address().clone()
    }

    fn origin(&mut self) -> pallet_contracts::Origin<T> {
        self.env.ext().caller()
    }
}
