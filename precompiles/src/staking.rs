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

use core::marker::PhantomData;

use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use frame_system::RawOrigin;
use pallet_evm::{
    AddressMapping, BalanceConverter, ExitError, PrecompileFailure, PrecompileHandle,
};
use precompile_utils::EvmResult;
use sp_core::{H256, U256};
use sp_runtime::traits::{Dispatchable, StaticLookup, UniqueSaturatedInto};
use subtensor_runtime_common::ProxyType;

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
        + pallet_evm::Config
        + pallet_subtensor::Config
        + pallet_proxy::Config<ProxyType = ProxyType>,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_proxy::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    const INDEX: u64 = 2053;
}

#[precompile_utils::precompile]
impl<R> StakingPrecompileV2<R>
where
    R: frame_system::Config
        + pallet_evm::Config
        + pallet_subtensor::Config
        + pallet_proxy::Config<ProxyType = ProxyType>,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_proxy::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
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
        let amount_staked = amount_rao.unique_saturated_into();
        let hotkey = R::AccountId::from(address.0);
        let netuid = try_u16_from_u256(netuid)?;
        let call = pallet_subtensor::Call::<R>::add_stake {
            hotkey,
            netuid,
            amount_staked,
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("removeStake(bytes32,uint256,uint256)")]
    fn remove_stake(
        handle: &mut impl PrecompileHandle,
        address: H256,
        amount_alpha: U256,
        netuid: U256,
    ) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let hotkey = R::AccountId::from(address.0);
        let netuid = try_u16_from_u256(netuid)?;
        let amount_unstaked = amount_alpha.unique_saturated_into();
        let call = pallet_subtensor::Call::<R>::remove_stake {
            hotkey,
            netuid,
            amount_unstaked,
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

        Ok(stake.into())
    }

    #[precompile::public("getTotalHotkeyStake(bytes32)")]
    #[precompile::view]
    fn get_total_hotkey_stake(
        _handle: &mut impl PrecompileHandle,
        hotkey: H256,
    ) -> EvmResult<U256> {
        let hotkey = R::AccountId::from(hotkey.0);
        let stake = pallet_subtensor::Pallet::<R>::get_total_stake_for_hotkey(&hotkey);

        Ok(stake.into())
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
            &hotkey, &coldkey, netuid,
        );

        Ok(stake.into())
    }

    #[precompile::public("addProxy(bytes32)")]
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
}

// Deprecated, exists for backward compatibility.
pub(crate) struct StakingPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for StakingPrecompile<R>
where
    R: frame_system::Config
        + pallet_evm::Config
        + pallet_subtensor::Config
        + pallet_proxy::Config<ProxyType = ProxyType>
        + pallet_balances::Config,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_proxy::Call<R>>
        + From<pallet_balances::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
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
        + pallet_subtensor::Config
        + pallet_proxy::Config<ProxyType = ProxyType>
        + pallet_balances::Config,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_proxy::Call<R>>
        + From<pallet_balances::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
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
        let call = pallet_subtensor::Call::<R>::add_stake {
            hotkey,
            netuid,
            amount_staked: amount_sub.unique_saturated_into(),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("removeStake(bytes32,uint256,uint256)")]
    fn remove_stake(
        handle: &mut impl PrecompileHandle,
        address: H256,
        amount: U256,
        netuid: U256,
    ) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let hotkey = R::AccountId::from(address.0);
        let netuid = try_u16_from_u256(netuid)?;
        let amount_unstaked =
            <R as pallet_evm::Config>::BalanceConverter::into_substrate_balance(amount)
                .ok_or(ExitError::OutOfFund)?;
        let amount_unstaked = amount_unstaked.unique_saturated_into();
        let call = pallet_subtensor::Call::<R>::remove_stake {
            hotkey,
            netuid,
            amount_unstaked,
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
        let total_stake = pallet_subtensor::Pallet::<R>::get_total_stake_for_coldkey(&coldkey);
        // Convert to EVM decimals
        let stake_u256 = U256::from(total_stake);
        let stake_eth = <R as pallet_evm::Config>::BalanceConverter::into_evm_balance(stake_u256)
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
        let total_stake = pallet_subtensor::Pallet::<R>::get_total_stake_for_hotkey(&hotkey);
        // Convert to EVM decimals
        let stake_u256 = U256::from(total_stake);
        let stake_eth = <R as pallet_evm::Config>::BalanceConverter::into_evm_balance(stake_u256)
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
            &hotkey, &coldkey, netuid,
        );
        let stake = U256::from(stake);
        let stake = <R as pallet_evm::Config>::BalanceConverter::into_evm_balance(stake)
            .ok_or(ExitError::InvalidRange)?;

        Ok(stake)
    }

    #[precompile::public("addProxy(bytes32)")]
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
        let amount_sub =
            <R as pallet_evm::Config>::BalanceConverter::into_substrate_balance(amount)
                .ok_or(ExitError::OutOfFund)?;

        // Create a transfer call from the smart contract to the caller
        let transfer_call = <R as frame_system::Config>::RuntimeCall::from(
            pallet_balances::Call::<R>::transfer_allow_death {
                dest: account_id.clone().into(),
                value: amount_sub.unique_saturated_into(),
            },
        );

        // Execute the transfer
        let transfer_result = transfer_call.dispatch(RawOrigin::Signed(Self::account_id()).into());

        if let Err(dispatch_error) = transfer_result {
            log::error!(
                "Transfer back to caller failed. Error: {:?}",
                dispatch_error
            );
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
