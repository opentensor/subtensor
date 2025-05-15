use core::marker::PhantomData;

use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use pallet_evm::PrecompileHandle;
use precompile_utils::{EvmResult, prelude::Address};
use sp_runtime::traits::{Dispatchable, StaticLookup};
use sp_std::vec::Vec;

use crate::PrecompileExt;

pub(crate) struct RuntimeCallQueryPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for RuntimeCallQueryPrecompile<R>
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
    const INDEX: u64 = 2055;
}
