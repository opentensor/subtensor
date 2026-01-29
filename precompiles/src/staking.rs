// The goal of staking precompile is to allow interaction between EVM users and smart contracts and
// subtensor staking functionality, namely add_stake, and remove_stake extrinsicsk, as well as the
// staking state.
//
// Additional requirement is to preserve compatibility with Ethereum indexers, which requires
// no balance transfers from EVM accounts without a corresponding transaction that can be
// parsed by an indexer.
//
// Implementation of add_stake:
//   - User transfers balance that will be staked to the precompile address with a payable
//     method addStake. This method also takes hotkey public key (bytes32) of the hotkey
//     that the stake should be assigned to.
//   - Precompile transfers the balance back to the signing address, and then invokes
//     do_add_stake from subtensor pallet with signing origin that mmatches to HashedAddressMapping
//     of the message sender, which will effectively withdraw and stake balance from the message
//     sender.
//   - Precompile checks the result of do_add_stake and, in case of a failure, reverts the transaction,
//     and leaves the balance on the message sender account.
//
// Implementation of remove_stake:
//   - User involkes removeStake method and specifies hotkey public key (bytes32) of the hotkey
//     to remove stake from, and the amount to unstake.
//   - Precompile calls do_remove_stake method of the subtensor pallet with the signing origin of message
//     sender, which effectively unstakes the specified amount and credits it to the message sender
//   - Precompile checks the result of do_remove_stake and, in case of a failure, reverts the transaction.
//

use alloc::vec::Vec;
use core::marker::PhantomData;
use frame_support::dispatch::{DispatchInfo, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::IsSubType;
use frame_system::RawOrigin;
use pallet_evm::{
    AddressMapping, BalanceConverter, EvmBalance, ExitError, PrecompileFailure, PrecompileHandle,
    SubstrateBalance,
};
use pallet_subtensor_proxy as pallet_proxy;
use precompile_utils::EvmResult;
use sp_core::{H256, U256};
use sp_runtime::traits::{AsSystemOriginSigner, Dispatchable, StaticLookup, UniqueSaturatedInto};
use sp_std::vec;
use subtensor_runtime_common::{Currency, NetUid, ProxyType};

use crate::{PrecompileExt, PrecompileHandleExt};

// Old StakingPrecompile had ETH-precision in values, which was not alligned with Substrate API. So
// it's kinda deprecated, but exists for backward compatibility. Eventually, we should remove it
// to stop supporting both precompiles.
//
// All the future extensions should happen in StakingPrecompileV2.
pub(crate) struct StakingPrecompileV2<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for StakingPrecompileV2<R>
where
    R: frame_system::Config
        + pallet_balances::Config
        + pallet_evm::Config
        + pallet_rate_limiting::Config<RuntimeCall = <R as frame_system::Config>::RuntimeCall>
        + pallet_subtensor::Config
        + pallet_proxy::Config<ProxyType = ProxyType>
        + pallet_shield::Config
        + pallet_subtensor_proxy::Config
        + Send
        + Sync
        + scale_info::TypeInfo,
    R::AccountId: From<[u8; 32]> + Into<[u8; 32]>,
    <R as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<R::AccountId> + Clone,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_proxy::Call<R>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<pallet_balances::Call<R>>
        + IsSubType<pallet_subtensor::Call<R>>
        + IsSubType<pallet_shield::Call<R>>
        + IsSubType<pallet_subtensor_proxy::Call<R>>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    const INDEX: u64 = 2053;
}

#[precompile_utils::precompile]
impl<R> StakingPrecompileV2<R>
where
    R: frame_system::Config
        + pallet_balances::Config
        + pallet_evm::Config
        + pallet_rate_limiting::Config<RuntimeCall = <R as frame_system::Config>::RuntimeCall>
        + pallet_subtensor::Config
        + pallet_proxy::Config<ProxyType = ProxyType>
        + pallet_shield::Config
        + pallet_subtensor_proxy::Config
        + Send
        + Sync
        + scale_info::TypeInfo,
    R::AccountId: From<[u8; 32]> + Into<[u8; 32]>,
    <R as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<R::AccountId> + Clone,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_proxy::Call<R>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<pallet_balances::Call<R>>
        + IsSubType<pallet_subtensor::Call<R>>
        + IsSubType<pallet_shield::Call<R>>
        + IsSubType<pallet_subtensor_proxy::Call<R>>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    #[precompile::public("addStake(bytes32,uint256,uint256)")]
    #[precompile::payable]
    fn add_stake(
        handle: &mut impl PrecompileHandle,
        address: H256,
        amount_rao: U256,
        netuid: U256,
    ) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let amount_staked: u64 = amount_rao.unique_saturated_into();
        let hotkey = R::AccountId::from(address.0);
        let netuid = try_u16_from_u256(netuid)?;
        let call = pallet_subtensor::Call::<R>::add_stake {
            hotkey,
            netuid: netuid.into(),
            amount_staked: amount_staked.into(),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("removeStake(bytes32,uint256,uint256)")]
    #[precompile::payable]
    fn remove_stake(
        handle: &mut impl PrecompileHandle,
        address: H256,
        amount_alpha: U256,
        netuid: U256,
    ) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let hotkey = R::AccountId::from(address.0);
        let netuid = try_u16_from_u256(netuid)?;
        let amount_unstaked: u64 = amount_alpha.unique_saturated_into();
        let call = pallet_subtensor::Call::<R>::remove_stake {
            hotkey,
            netuid: netuid.into(),
            amount_unstaked: amount_unstaked.into(),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    fn call_remove_stake_full_limit(
        handle: &mut impl PrecompileHandle,
        hotkey: H256,
        netuid: U256,
        limit_price: Option<u64>,
    ) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let hotkey = R::AccountId::from(hotkey.0);
        let netuid = try_u16_from_u256(netuid)?;
        let call = pallet_subtensor::Call::<R>::remove_stake_full_limit {
            hotkey,
            netuid: netuid.into(),
            limit_price: limit_price.map(Into::into),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("removeStakeFull(bytes32,uint256)")]
    #[precompile::payable]
    fn remove_stake_full(
        handle: &mut impl PrecompileHandle,
        hotkey: H256,
        netuid: U256,
    ) -> EvmResult<()> {
        Self::call_remove_stake_full_limit(handle, hotkey, netuid, None)
    }

    #[precompile::public("removeStakeFullLimit(bytes32,uint256,uint256)")]
    #[precompile::payable]
    fn remove_stake_full_limit(
        handle: &mut impl PrecompileHandle,
        hotkey: H256,
        netuid: U256,
        limit_price: U256,
    ) -> EvmResult<()> {
        let limit_price = try_u64_from_u256(limit_price)?;
        Self::call_remove_stake_full_limit(handle, hotkey, netuid, Some(limit_price))
    }

    #[precompile::public("moveStake(bytes32,bytes32,uint256,uint256,uint256)")]
    #[precompile::payable]
    fn move_stake(
        handle: &mut impl PrecompileHandle,
        origin_hotkey: H256,
        destination_hotkey: H256,
        origin_netuid: U256,
        destination_netuid: U256,
        amount_alpha: U256,
    ) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let origin_hotkey = R::AccountId::from(origin_hotkey.0);
        let destination_hotkey = R::AccountId::from(destination_hotkey.0);
        let origin_netuid = try_u16_from_u256(origin_netuid)?;
        let destination_netuid = try_u16_from_u256(destination_netuid)?;
        let alpha_amount: u64 = amount_alpha.unique_saturated_into();
        let call = pallet_subtensor::Call::<R>::move_stake {
            origin_hotkey,
            destination_hotkey,
            origin_netuid: origin_netuid.into(),
            destination_netuid: destination_netuid.into(),
            alpha_amount: alpha_amount.into(),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("transferStake(bytes32,bytes32,uint256,uint256,uint256)")]
    #[precompile::payable]
    fn transfer_stake(
        handle: &mut impl PrecompileHandle,
        destination_coldkey: H256,
        hotkey: H256,
        origin_netuid: U256,
        destination_netuid: U256,
        amount_alpha: U256,
    ) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let destination_coldkey = R::AccountId::from(destination_coldkey.0);
        let hotkey = R::AccountId::from(hotkey.0);
        let origin_netuid = try_u16_from_u256(origin_netuid)?;
        let destination_netuid = try_u16_from_u256(destination_netuid)?;
        let alpha_amount: u64 = amount_alpha.unique_saturated_into();
        let call = pallet_subtensor::Call::<R>::transfer_stake {
            destination_coldkey,
            hotkey,
            origin_netuid: origin_netuid.into(),
            destination_netuid: destination_netuid.into(),
            alpha_amount: alpha_amount.into(),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("burnAlpha(bytes32,uint256,uint256)")]
    #[precompile::payable]
    fn burn_alpha(
        handle: &mut impl PrecompileHandle,
        hotkey: H256,
        amount: U256,
        netuid: U256,
    ) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let hotkey = R::AccountId::from(hotkey.0);
        let netuid = try_u16_from_u256(netuid)?;
        let amount: u64 = amount.unique_saturated_into();
        let call = pallet_subtensor::Call::<R>::burn_alpha {
            hotkey,
            amount: amount.into(),
            netuid: netuid.into(),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("getTotalColdkeyStake(bytes32)")]
    #[precompile::view]
    fn get_total_coldkey_stake(
        _handle: &mut impl PrecompileHandle,
        coldkey: H256,
    ) -> EvmResult<U256> {
        let coldkey = R::AccountId::from(coldkey.0);
        let stake = pallet_subtensor::Pallet::<R>::get_total_stake_for_coldkey(&coldkey);

        Ok(stake.to_u64().into())
    }

    #[precompile::public("getTotalHotkeyStake(bytes32)")]
    #[precompile::view]
    fn get_total_hotkey_stake(
        _handle: &mut impl PrecompileHandle,
        hotkey: H256,
    ) -> EvmResult<U256> {
        let hotkey = R::AccountId::from(hotkey.0);
        let stake = pallet_subtensor::Pallet::<R>::get_total_stake_for_hotkey(&hotkey);

        Ok(stake.to_u64().into())
    }

    #[precompile::public("getStake(bytes32,bytes32,uint256)")]
    #[precompile::view]
    fn get_stake(
        _: &mut impl PrecompileHandle,
        hotkey: H256,
        coldkey: H256,
        netuid: U256,
    ) -> EvmResult<U256> {
        let hotkey = R::AccountId::from(hotkey.0);
        let coldkey = R::AccountId::from(coldkey.0);
        let netuid = try_u16_from_u256(netuid)?;
        let stake = pallet_subtensor::Pallet::<R>::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid.into(),
        );

        Ok(u64::from(stake).into())
    }

    #[precompile::public("getAlphaStakedValidators(bytes32,uint256)")]
    #[precompile::view]
    fn get_alpha_staked_validators(
        _handle: &mut impl PrecompileHandle,
        hotkey: H256,
        netuid: U256,
    ) -> EvmResult<Vec<H256>> {
        let hotkey = R::AccountId::from(hotkey.0);
        let mut coldkeys: Vec<H256> = vec![];
        let netuid = NetUid::from(try_u16_from_u256(netuid)?);
        for ((coldkey, netuid_in_alpha), _) in pallet_subtensor::Alpha::<R>::iter_prefix((hotkey,))
        {
            if netuid == netuid_in_alpha {
                let key: [u8; 32] = coldkey.into();
                coldkeys.push(key.into());
            }
        }

        Ok(coldkeys)
    }

    #[precompile::public("getTotalAlphaStaked(bytes32,uint256)")]
    #[precompile::view]
    fn get_total_alpha_staked(
        _handle: &mut impl PrecompileHandle,
        hotkey: H256,
        netuid: U256,
    ) -> EvmResult<U256> {
        let hotkey = R::AccountId::from(hotkey.0);
        let netuid = try_u16_from_u256(netuid)?;
        let stake =
            pallet_subtensor::Pallet::<R>::get_stake_for_hotkey_on_subnet(&hotkey, netuid.into());

        Ok(u64::from(stake).into())
    }

    #[precompile::public("getNominatorMinRequiredStake()")]
    #[precompile::view]
    fn get_nominator_min_required_stake(_handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
        let stake = pallet_subtensor::Pallet::<R>::get_nominator_min_required_stake();

        Ok(stake.into())
    }

    #[precompile::public("addProxy(bytes32)")]
    #[precompile::payable]
    fn add_proxy(handle: &mut impl PrecompileHandle, delegate: H256) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let delegate = R::AccountId::from(delegate.0);
        let delegate = <R as frame_system::Config>::Lookup::unlookup(delegate);
        let call = pallet_proxy::Call::<R>::add_proxy {
            delegate,
            proxy_type: ProxyType::Staking,
            delay: 0u32.into(),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("removeProxy(bytes32)")]
    #[precompile::payable]
    fn remove_proxy(handle: &mut impl PrecompileHandle, delegate: H256) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let delegate = R::AccountId::from(delegate.0);
        let delegate = <R as frame_system::Config>::Lookup::unlookup(delegate);
        let call = pallet_proxy::Call::<R>::remove_proxy {
            delegate,
            proxy_type: ProxyType::Staking,
            delay: 0u32.into(),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("addStakeLimit(bytes32,uint256,uint256,bool,uint256)")]
    #[precompile::payable]
    fn add_stake_limit(
        handle: &mut impl PrecompileHandle,
        address: H256,
        amount_rao: U256,
        limit_price_rao: U256,
        allow_partial: bool,
        netuid: U256,
    ) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let amount_staked: u64 = amount_rao.unique_saturated_into();
        let limit_price: u64 = limit_price_rao.unique_saturated_into();
        let hotkey = R::AccountId::from(address.0);
        let netuid = try_u16_from_u256(netuid)?;
        let call = pallet_subtensor::Call::<R>::add_stake_limit {
            hotkey,
            netuid: netuid.into(),
            amount_staked: amount_staked.into(),
            limit_price: limit_price.into(),
            allow_partial,
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("removeStakeLimit(bytes32,uint256,uint256,bool,uint256)")]
    #[precompile::payable]
    fn remove_stake_limit(
        handle: &mut impl PrecompileHandle,
        address: H256,
        amount_alpha: U256,
        limit_price_rao: U256,
        allow_partial: bool,
        netuid: U256,
    ) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let hotkey = R::AccountId::from(address.0);
        let netuid = try_u16_from_u256(netuid)?;
        let amount_unstaked: u64 = amount_alpha.unique_saturated_into();
        let limit_price: u64 = limit_price_rao.unique_saturated_into();
        let call = pallet_subtensor::Call::<R>::remove_stake_limit {
            hotkey,
            netuid: netuid.into(),
            amount_unstaked: amount_unstaked.into(),
            limit_price: limit_price.into(),
            allow_partial,
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("getTotalColdkeyStakeOnSubnet(bytes32,uint256)")]
    #[precompile::view]
    fn get_total_coldkey_stake_on_subnet(
        _handle: &mut impl PrecompileHandle,
        coldkey: H256,
        netuid: U256,
    ) -> EvmResult<U256> {
        let coldkey = R::AccountId::from(coldkey.0);
        let netuid = try_u16_from_u256(netuid)?;
        let stake = pallet_subtensor::Pallet::<R>::get_total_stake_for_coldkey_on_subnet(
            &coldkey,
            netuid.into(),
        );

        Ok(stake.to_u64().into())
    }
}

// Deprecated, exists for backward compatibility.
pub(crate) struct StakingPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for StakingPrecompile<R>
where
    R: frame_system::Config
        + pallet_evm::Config
        + pallet_rate_limiting::Config<RuntimeCall = <R as frame_system::Config>::RuntimeCall>
        + pallet_subtensor::Config
        + pallet_proxy::Config<ProxyType = ProxyType>
        + pallet_balances::Config
        + pallet_shield::Config
        + pallet_subtensor_proxy::Config
        + Send
        + Sync
        + scale_info::TypeInfo,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<R::AccountId> + Clone,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_proxy::Call<R>>
        + From<pallet_balances::Call<R>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<pallet_balances::Call<R>>
        + IsSubType<pallet_subtensor::Call<R>>
        + IsSubType<pallet_shield::Call<R>>
        + IsSubType<pallet_subtensor_proxy::Call<R>>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <R as pallet_balances::Config>::Balance: TryFrom<U256>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    const INDEX: u64 = 2049;
}

#[precompile_utils::precompile]
impl<R> StakingPrecompile<R>
where
    R: frame_system::Config
        + pallet_evm::Config
        + pallet_rate_limiting::Config<RuntimeCall = <R as frame_system::Config>::RuntimeCall>
        + pallet_subtensor::Config
        + pallet_proxy::Config<ProxyType = ProxyType>
        + pallet_balances::Config
        + pallet_shield::Config
        + pallet_subtensor_proxy::Config
        + Send
        + Sync
        + scale_info::TypeInfo,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<R::AccountId> + Clone,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_proxy::Call<R>>
        + From<pallet_balances::Call<R>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<pallet_balances::Call<R>>
        + IsSubType<pallet_subtensor::Call<R>>
        + IsSubType<pallet_shield::Call<R>>
        + IsSubType<pallet_subtensor_proxy::Call<R>>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <R as pallet_balances::Config>::Balance: TryFrom<U256>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    #[precompile::public("addStake(bytes32,uint256)")]
    #[precompile::payable]
    fn add_stake(handle: &mut impl PrecompileHandle, address: H256, netuid: U256) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let amount = handle.context().apparent_value;

        if !amount.is_zero() {
            Self::transfer_back_to_caller(&account_id, amount)?;
        }

        let amount_sub = handle.try_convert_apparent_value::<R>()?;
        let hotkey = R::AccountId::from(address.0);
        let netuid = try_u16_from_u256(netuid)?;
        let amount_staked: u64 = amount_sub.unique_saturated_into();
        let call = pallet_subtensor::Call::<R>::add_stake {
            hotkey,
            netuid: netuid.into(),
            amount_staked: amount_staked.into(),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("removeStake(bytes32,uint256,uint256)")]
    #[precompile::payable]
    fn remove_stake(
        handle: &mut impl PrecompileHandle,
        address: H256,
        amount: U256,
        netuid: U256,
    ) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let hotkey = R::AccountId::from(address.0);
        let netuid = try_u16_from_u256(netuid)?;
        let amount = EvmBalance::new(amount);
        let amount_unstaked =
            <R as pallet_evm::Config>::BalanceConverter::into_substrate_balance(amount)
                .map(|amount| amount.into_u64_saturating())
                .ok_or(ExitError::OutOfFund)?;
        let call = pallet_subtensor::Call::<R>::remove_stake {
            hotkey,
            netuid: netuid.into(),
            amount_unstaked: amount_unstaked.into(),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("getTotalColdkeyStake(bytes32)")]
    #[precompile::view]
    fn get_total_coldkey_stake(
        _handle: &mut impl PrecompileHandle,
        coldkey: H256,
    ) -> EvmResult<U256> {
        let coldkey = R::AccountId::from(coldkey.0);

        // get total stake of coldkey
        let total_stake =
            pallet_subtensor::Pallet::<R>::get_total_stake_for_coldkey(&coldkey).to_u64();
        // Convert to EVM decimals
        let stake_u256: SubstrateBalance = total_stake.into();
        let stake_eth = <R as pallet_evm::Config>::BalanceConverter::into_evm_balance(stake_u256)
            .map(|amount| amount.into_u256())
            .ok_or(ExitError::InvalidRange)?;

        Ok(stake_eth)
    }

    #[precompile::public("getTotalHotkeyStake(bytes32)")]
    #[precompile::view]
    fn get_total_hotkey_stake(
        _handle: &mut impl PrecompileHandle,
        hotkey: H256,
    ) -> EvmResult<U256> {
        let hotkey = R::AccountId::from(hotkey.0);

        // get total stake of hotkey
        let total_stake =
            pallet_subtensor::Pallet::<R>::get_total_stake_for_hotkey(&hotkey).to_u64();
        // Convert to EVM decimals
        let stake_u256: SubstrateBalance = total_stake.into();
        let stake_eth = <R as pallet_evm::Config>::BalanceConverter::into_evm_balance(stake_u256)
            .map(|amount| amount.into_u256())
            .ok_or(ExitError::InvalidRange)?;

        Ok(stake_eth)
    }

    #[precompile::public("getStake(bytes32,bytes32,uint256)")]
    #[precompile::view]
    fn get_stake(
        _: &mut impl PrecompileHandle,
        hotkey: H256,
        coldkey: H256,
        netuid: U256,
    ) -> EvmResult<U256> {
        let hotkey = R::AccountId::from(hotkey.0);
        let coldkey = R::AccountId::from(coldkey.0);
        let netuid = try_u16_from_u256(netuid)?;
        let stake = pallet_subtensor::Pallet::<R>::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid.into(),
        );
        let stake: SubstrateBalance = u64::from(stake).into();
        let stake = <R as pallet_evm::Config>::BalanceConverter::into_evm_balance(stake)
            .map(|amount| amount.into_u256())
            .ok_or(ExitError::InvalidRange)?;

        Ok(stake)
    }

    #[precompile::public("addProxy(bytes32)")]
    #[precompile::payable]
    fn add_proxy(handle: &mut impl PrecompileHandle, delegate: H256) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let delegate = R::AccountId::from(delegate.0);
        let delegate = <R as frame_system::Config>::Lookup::unlookup(delegate);
        let call = pallet_proxy::Call::<R>::add_proxy {
            delegate,
            proxy_type: ProxyType::Staking,
            delay: 0u32.into(),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("removeProxy(bytes32)")]
    #[precompile::payable]
    fn remove_proxy(handle: &mut impl PrecompileHandle, delegate: H256) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let delegate = R::AccountId::from(delegate.0);
        let delegate = <R as frame_system::Config>::Lookup::unlookup(delegate);
        let call = pallet_proxy::Call::<R>::remove_proxy {
            delegate,
            proxy_type: ProxyType::Staking,
            delay: 0u32.into(),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    fn transfer_back_to_caller(
        account_id: &<R as frame_system::Config>::AccountId,
        amount: U256,
    ) -> Result<(), PrecompileFailure> {
        let amount = EvmBalance::new(amount);
        let amount_sub =
            <R as pallet_evm::Config>::BalanceConverter::into_substrate_balance(amount)
                .ok_or(ExitError::OutOfFund)?;

        // Create a transfer call from the smart contract to the caller
        let value = amount_sub
            .into_u64_saturating()
            .try_into()
            .map_err(|_| ExitError::Other("Failed to convert u64 to Balance".into()))?;
        let transfer_call = <R as frame_system::Config>::RuntimeCall::from(
            pallet_balances::Call::<R>::transfer_allow_death {
                dest: account_id.clone().into(),
                value,
            },
        );

        // Execute the transfer
        let transfer_result = transfer_call.dispatch(RawOrigin::Signed(Self::account_id()).into());

        if let Err(dispatch_error) = transfer_result {
            log::error!("Transfer back to caller failed. Error: {dispatch_error:?}");
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("Transfer back to caller failed".into()),
            });
        }

        Ok(())
    }
}

fn try_u16_from_u256(value: U256) -> Result<u16, PrecompileFailure> {
    value.try_into().map_err(|_| PrecompileFailure::Error {
        exit_status: ExitError::Other("the value is outside of u16 bounds".into()),
    })
}

fn try_u64_from_u256(value: U256) -> Result<u64, PrecompileFailure> {
    value.try_into().map_err(|_| PrecompileFailure::Error {
        exit_status: ExitError::Other("the value is outside of u64 bounds".into()),
    })
}
