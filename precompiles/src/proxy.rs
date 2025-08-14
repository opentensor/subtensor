use core::marker::PhantomData;

use crate::{PrecompileExt, PrecompileHandleExt};
use codec::DecodeLimit;
use fp_evm::{ExitError, PrecompileFailure};
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use frame_system::RawOrigin;
use pallet_evm::{AddressMapping, PrecompileHandle};
use precompile_utils::EvmResult;
use sp_core::H256;
use sp_runtime::traits::{Dispatchable, StaticLookup};
use sp_std::boxed::Box;
use sp_std::vec::Vec;
use subtensor_runtime_common::ProxyType;
pub struct ProxyPrecompile<R>(PhantomData<R>);
const MAX_DECODE_DEPTH: u32 = 8;

impl<R> PrecompileExt<R::AccountId> for ProxyPrecompile<R>
where
    R: frame_system::Config
        + pallet_evm::Config
        + pallet_subtensor::Config
        + pallet_proxy::Config<ProxyType = ProxyType>,
    R::AccountId: From<[u8; 32]> + Into<[u8; 32]>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
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
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_proxy::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    #[precompile::public("createPureProxy()")]
    #[precompile::payable]
    pub fn create_pure_proxy(handle: &mut impl PrecompileHandle) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();

        let proxy_type: ProxyType = ProxyType::Any;
        let delay = 0u32.into();
        let index = 0u16.into();

        let call = pallet_proxy::Call::<R>::create_evm_pure {
            proxy_type,
            delay,
            index,
            evm_address: handle.context().caller,
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("killPureProxy(bytes32)")]
    #[precompile::payable]
    pub fn kill_pure_proxy(handle: &mut impl PrecompileHandle, proxy: H256) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();

        let call = pallet_proxy::Call::<R>::kill_evm_pure {
            proxy: proxy.0.into(),
            evm_address: handle.context().caller,
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("proxyCall(bytes32,uint8[])")]
    #[precompile::payable]
    pub fn proxy_call(
        handle: &mut impl PrecompileHandle,
        real: H256,
        call: Vec<u8>,
    ) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();

        let call = <R as pallet_proxy::Config>::RuntimeCall::decode_with_depth_limit(
            MAX_DECODE_DEPTH,
            &mut &call[..],
        )
        .map_err(|_| PrecompileFailure::Error {
            exit_status: ExitError::Other("The raw call data not correctly encoded".into()),
        })?;

        let proxy_type: ProxyType = ProxyType::Any;
        let call = pallet_proxy::Call::<R>::evm_proxy {
            force_proxy_type: Some(proxy_type),
            call: Box::new(call),
            evm_address: handle.context().caller,
            proxy: real.0.into(),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }
}
