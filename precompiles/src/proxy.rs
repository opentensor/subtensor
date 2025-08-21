// The goal of the proxy precompile is to allow smart contracts to interact with the subtensor
// pallet by making other precompile calls on behalf of users. Much like DEXs can swap tokens
// on behalf of users using ERC20's allowance functionality, the proxy precompile allows smart
// contracts to perform actions on behalf of users, such as transferring stake.
//

use alloc::vec::Vec;
use core::marker::PhantomData;
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use frame_system::RawOrigin;
use pallet_evm::{
    AddressMapping, BalanceConverter, EvmBalance, ExitError, PrecompileFailure, PrecompileHandle,
    SubstrateBalance,
};
use precompile_utils::EvmResult;
use sp_core::{H256, U256};
use sp_runtime::traits::{Dispatchable, StaticLookup, UniqueSaturatedInto};
use sp_std::vec;
use subtensor_runtime_common::{Currency, NetUid, ProxyType};

use crate::{PrecompileExt, PrecompileHandleExt};

// Old StakingPrecompile had ETH-precision in values, which was not alligned with Substrate API. So
// it's kinda deprecated, but exists for backward compatibility. Eventually, we should remove it
// to stop supporting both precompiles.
//
// All the future extensions should happen in StakingPrecompileV2.
pub(crate) struct ProxyPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for ProxyPrecompile<R>
where
    R: frame_system::Config
        + pallet_evm::Config
        + pallet_subtensor::Config
        + pallet_proxy::Config<ProxyType = ProxyType>,
    R::AccountId: From<[u8; 32]> + Into<[u8; 32]>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_proxy::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    const INDEX: u64 = 2059;
}

#[precompile_utils::precompile]
impl<R> ProxyPrecompile<R>
where
    R: frame_system::Config
        + pallet_evm::Config
        + pallet_subtensor::Config
        + pallet_proxy::Config<ProxyType = ProxyType>,
    R::AccountId: From<[u8; 32]> + Into<[u8; 32]>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_proxy::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    #[precompile::public("proxy(bytes32,uint256,bytes)")]
    #[precompile::payable]
    fn proxy(
        handle: &mut impl PrecompileHandle,
        proxied_account: H256,
        force_proxy_type: U256,
        call: Vec<u8>,
    ) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let real = R::AccountId::from(proxied_account.0);
        let proxy_call = pallet_proxy::Call::<R>::proxy {
            real,
            force_proxy_type: ProxyType::try_from(force_proxy_type).map_err(|_| {
                PrecompileFailure::Error {
                    exit_status: ExitError::Other("Invalid proxy type".into()),
                }
            })?,
            call: call.into(),
        };

        handle.try_dispatch_runtime_call::<R, _>(proxy_call, RawOrigin::Signed(account_id))
    }
}
