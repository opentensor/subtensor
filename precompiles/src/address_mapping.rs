extern crate alloc;
use core::marker::PhantomData;
use pallet_evm::AddressMapping;

use crate::PrecompileExt;
use sp_core::{ByteArray, H256};

use frame_support::dispatch::{DispatchInfo, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::IsSubType;
use pallet_evm::PrecompileHandle;
use pallet_subtensor_proxy as pallet_proxy;
use precompile_utils::EvmResult;
use precompile_utils::prelude::Address;
use sp_runtime::traits::{AsSystemOriginSigner, Dispatchable};

pub struct AddressMappingPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for AddressMappingPrecompile<R>
where
    R: frame_system::Config
        + pallet_balances::Config
        + pallet_crowdloan::Config
        + pallet_evm::Config
        + pallet_proxy::Config
        + pallet_subtensor::Config
        + Send
        + Sync
        + scale_info::TypeInfo,
    R::AccountId: From<[u8; 32]> + ByteArray + Into<[u8; 32]>,
    <R as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<R::AccountId> + Clone,
    <R as frame_system::Config>::RuntimeCall: From<pallet_crowdloan::Call<R>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<pallet_balances::Call<R>>
        + IsSubType<pallet_subtensor::Call<R>>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_crowdloan::Call<R>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
{
    const INDEX: u64 = 2060;
}

#[precompile_utils::precompile]
impl<R> AddressMappingPrecompile<R>
where
    R: frame_system::Config
        + pallet_balances::Config
        + pallet_crowdloan::Config
        + pallet_evm::Config
        + pallet_proxy::Config
        + pallet_subtensor::Config
        + Send
        + Sync
        + scale_info::TypeInfo,
    R::AccountId: From<[u8; 32]> + ByteArray + Into<[u8; 32]>,
    <R as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<R::AccountId> + Clone,
    <R as frame_system::Config>::RuntimeCall: From<pallet_crowdloan::Call<R>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<pallet_balances::Call<R>>
        + IsSubType<pallet_subtensor::Call<R>>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_crowdloan::Call<R>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
{
    #[precompile::public("addressMapping(address)")]
    #[precompile::view]
    fn address_mapping(
        _handle: &mut impl PrecompileHandle,
        target_address: Address,
    ) -> EvmResult<H256> {
        let target_address: [u8; 32] = R::AddressMapping::into_account_id(target_address.0).into();
        Ok(target_address.into())
    }
}
