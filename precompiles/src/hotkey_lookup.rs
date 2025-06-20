use core::marker::PhantomData;

use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use pallet_evm::PrecompileHandle;
use precompile_utils::{EvmResult, prelude::Address};
use sp_runtime::traits::{Dispatchable, StaticLookup};
use sp_std::vec::Vec;
use parity_scale_codec::Encode;

use crate::PrecompileExt;

pub(crate) struct HotkeyLookupPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for HotkeyLookupPrecompile<R>
where
    R: frame_system::Config + pallet_subtensor::Config + pallet_evm::Config,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeCall:
        GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    const INDEX: u64 = 2054;
}

#[precompile_utils::precompile]
impl<R> HotkeyLookupPrecompile<R>
where
    R: frame_system::Config + pallet_subtensor::Config + pallet_evm::Config,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeCall:
        GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    #[precompile::public("hotkeyLookup(uint16,address,uint16)")]
    #[precompile::view]
    fn hotkey_lookup(
        _handle: &mut impl PrecompileHandle,
        netuid: u16,
        evm_address: Address,
        limit: u16,
    ) -> EvmResult<Vec<(Address, u64)>> {
        let results = pallet_subtensor::Pallet::<R>::hotkey_lookup(
            netuid.into(),
            evm_address.0,
            limit,
        );
        
        // Convert AccountId to Address for EVM compatibility
        Ok(results.into_iter()
            .map(|(account_id, block)| {
                let mut bytes = [0u8; 20];
                let account_bytes = account_id.encode();
                bytes.copy_from_slice(&account_bytes[..20.min(account_bytes.len())]);
                (Address::from(bytes), block)
            })
            .collect())
    }
}
