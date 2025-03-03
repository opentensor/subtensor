use core::marker::PhantomData;

use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use pallet_evm::PrecompileHandle;
use precompile_utils::EvmResult;
use sp_core::{H256, U256};
use sp_runtime::traits::{Dispatchable, StaticLookup, UniqueSaturatedInto};

use crate::{PrecompileExt, PrecompileHandleExt, contract_to_origin, parse_pubkey};

pub(crate) struct BalanceTransferPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt for BalanceTransferPrecompile<R>
where
    R: frame_system::Config + pallet_balances::Config + pallet_evm::Config,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeCall:
        GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_balances::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
    <R as pallet_balances::Config>::Balance: From<U256>,
{
    const INDEX: u64 = 2048;
    const ADDRESS_SS58: [u8; 32] = [
        0x07, 0xec, 0x71, 0x2a, 0x5d, 0x38, 0x43, 0x4d, 0xdd, 0x03, 0x3f, 0x8f, 0x02, 0x4e, 0xcd,
        0xfc, 0x4b, 0xb5, 0x95, 0x1c, 0x13, 0xc3, 0x08, 0x5c, 0x39, 0x9c, 0x8a, 0x5f, 0x62, 0x93,
        0x70, 0x5d,
    ];
}

#[precompile_utils::precompile]
impl<R> BalanceTransferPrecompile<R>
where
    R: frame_system::Config + pallet_balances::Config + pallet_evm::Config,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeCall:
        GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_balances::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
    <R as pallet_balances::Config>::Balance: From<U256>,
{
    #[precompile::public("transfer(bytes32)")]
    #[precompile::payable]
    fn transfer(handle: &mut impl PrecompileHandle, address: H256) -> EvmResult<()> {
        let amount_sub = handle.try_convert_apparent_value::<R>()?;

        if amount_sub.is_zero() {
            return Ok(());
        }

        let dest = parse_pubkey::<R::AccountId>(address.as_bytes())?.0.into();

        let call = pallet_balances::Call::<R>::transfer_allow_death {
            dest,
            value: amount_sub.unique_saturated_into(),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, contract_to_origin(&Self::ADDRESS_SS58)?)
    }
}
