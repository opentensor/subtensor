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
pub struct PureProxyPrecompile<R>(PhantomData<R>);
const MAX_DECODE_DEPTH: u32 = 8;

impl<R> PrecompileExt<R::AccountId> for PureProxyPrecompile<R>
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
impl<R> PureProxyPrecompile<R>
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
    pub fn create_pure_proxy(handle: &mut impl PrecompileHandle) -> EvmResult<H256> {
        let account_id = handle.caller_account_id::<R>();

        let proxy_type: ProxyType = ProxyType::Any;
        let delay = 0u32.into();
        let index = 0u16.into();

        let call = pallet_proxy::Call::<R>::create_pure {
            proxy_type,
            delay,
            index,
        };

        let pure_account =
            pallet_proxy::Pallet::<R>::pure_account(&account_id, &proxy_type, index, None);

        let data = pallet_proxy::Proxies::<R>::get(&pure_account);
        if data.0.len() > 0 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("Pure proxy already exists".into()),
            });
        }

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))?;

        Ok(H256::from(pure_account.into()))
    }

    #[precompile::public("pureProxyCall(bytes32,uint8[])")]
    #[precompile::payable]
    pub fn pure_proxy_call(
        handle: &mut impl PrecompileHandle,
        real: H256,
        call: Vec<u8>,
    ) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let real = R::Lookup::unlookup(real.0.into());

        let call = <R as pallet_proxy::Config>::RuntimeCall::decode_with_depth_limit(
            MAX_DECODE_DEPTH,
            &mut &call[..],
        )
        .map_err(|_| PrecompileFailure::Error {
            exit_status: ExitError::Other("The raw call data not correctly encoded".into()),
        })?;

        let proxy_type: ProxyType = ProxyType::Any;
        let call = pallet_proxy::Call::<R>::proxy {
            real,
            force_proxy_type: Some(proxy_type),
            call: Box::new(call),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }
}
