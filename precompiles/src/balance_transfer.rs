use core::marker::PhantomData;

use frame_support::dispatch::{DispatchInfo, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::IsSubType;
use frame_system::RawOrigin;
use pallet_evm::PrecompileHandle;
use precompile_utils::EvmResult;
use sp_core::{H256, U256};
use sp_runtime::traits::{AsSystemOriginSigner, Dispatchable, StaticLookup, UniqueSaturatedInto};

use crate::{PrecompileExt, PrecompileHandleExt};

pub(crate) struct BalanceTransferPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for BalanceTransferPrecompile<R>
where
    R: frame_system::Config
        + pallet_balances::Config
        + pallet_evm::Config
        + pallet_rate_limiting::Config<RuntimeCall = <R as frame_system::Config>::RuntimeCall>
        + pallet_subtensor::Config
        + pallet_shield::Config
        + pallet_subtensor_proxy::Config
        + Send
        + Sync
        + scale_info::TypeInfo,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<R::AccountId> + Clone,
    <R as frame_system::Config>::RuntimeCall: GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<pallet_balances::Call<R>>
        + IsSubType<pallet_subtensor::Call<R>>
        + IsSubType<pallet_shield::Call<R>>
        + IsSubType<pallet_subtensor_proxy::Call<R>>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_balances::Call<R>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
    <R as pallet_balances::Config>::Balance: TryFrom<U256>,
{
    const INDEX: u64 = 2048;
}

#[precompile_utils::precompile]
impl<R> BalanceTransferPrecompile<R>
where
    R: frame_system::Config
        + pallet_balances::Config
        + pallet_evm::Config
        + pallet_rate_limiting::Config<RuntimeCall = <R as frame_system::Config>::RuntimeCall>
        + pallet_subtensor::Config
        + pallet_shield::Config
        + pallet_subtensor_proxy::Config
        + Send
        + Sync
        + scale_info::TypeInfo,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<R::AccountId> + Clone,
    <R as frame_system::Config>::RuntimeCall: GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<pallet_balances::Call<R>>
        + IsSubType<pallet_subtensor::Call<R>>
        + IsSubType<pallet_shield::Call<R>>
        + IsSubType<pallet_subtensor_proxy::Call<R>>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_balances::Call<R>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
    <R as pallet_balances::Config>::Balance: TryFrom<U256>,
{
    #[precompile::public("transfer(bytes32)")]
    #[precompile::payable]
    fn transfer(handle: &mut impl PrecompileHandle, address: H256) -> EvmResult<()> {
        let amount_sub = handle.try_convert_apparent_value::<R>()?;

        if amount_sub.is_zero() {
            return Ok(());
        }

        let dest = R::AccountId::from(address.0).into();

        let call = pallet_balances::Call::<R>::transfer_allow_death {
            dest,
            value: amount_sub.unique_saturated_into(),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(Self::account_id()))
    }
}
