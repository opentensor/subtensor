use alloc::string::String;
use core::marker::PhantomData;

use fp_evm::{ExitError, PrecompileFailure};
use frame_support::dispatch::{DispatchInfo, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::IsSubType;
use frame_system::RawOrigin;
use pallet_evm::AddressMapping;
use pallet_evm::PrecompileHandle;
use pallet_subtensor_proxy as pallet_proxy;
use precompile_utils::prelude::Address;
use precompile_utils::{EvmResult, solidity::Codec};
use sp_core::{ByteArray, H256};
use sp_runtime::traits::{AsSystemOriginSigner, Dispatchable, UniqueSaturatedInto};

use crate::{PrecompileExt, PrecompileHandleExt};

pub struct CrowdloanPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for CrowdloanPrecompile<R>
where
    R: frame_system::Config
        + pallet_balances::Config
        + pallet_crowdloan::Config
        + pallet_evm::Config
        + pallet_proxy::Config
        + pallet_rate_limiting::Config<RuntimeCall = <R as frame_system::Config>::RuntimeCall>
        + pallet_subtensor::Config
        + Send
        + Sync
        + scale_info::TypeInfo,
    R::AccountId: From<[u8; 32]> + ByteArray,
    <R as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<R::AccountId> + Clone,
    <R as frame_system::Config>::RuntimeCall: From<pallet_crowdloan::Call<R>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<pallet_balances::Call<R>>
        + IsSubType<pallet_subtensor::Call<R>>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
{
    const INDEX: u64 = 2057;
}

#[precompile_utils::precompile]
impl<R> CrowdloanPrecompile<R>
where
    R: frame_system::Config
        + pallet_balances::Config
        + pallet_crowdloan::Config
        + pallet_evm::Config
        + pallet_proxy::Config
        + pallet_rate_limiting::Config<RuntimeCall = <R as frame_system::Config>::RuntimeCall>
        + pallet_subtensor::Config
        + Send
        + Sync
        + scale_info::TypeInfo,
    R::AccountId: From<[u8; 32]> + ByteArray,
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
    #[precompile::public("getCrowdloan(uint32)")]
    #[precompile::view]
    fn get_crowdloan(
        _handle: &mut impl PrecompileHandle,
        crowdloan_id: u32,
    ) -> EvmResult<CrowdloanInfo> {
        let crowdloan = pallet_crowdloan::Crowdloans::<R>::get(crowdloan_id).ok_or(
            PrecompileFailure::Error {
                exit_status: ExitError::Other("Crowdloan not found".into()),
            },
        )?;

        Ok(CrowdloanInfo {
            creator: H256::from_slice(crowdloan.creator.as_slice()),
            deposit: crowdloan.deposit,
            min_contribution: crowdloan.min_contribution,
            end: crowdloan.end.unique_saturated_into(),
            cap: crowdloan.cap,
            funds_account: H256::from_slice(crowdloan.funds_account.as_slice()),
            raised: crowdloan.raised,
            has_target_address: crowdloan.target_address.is_some(),
            target_address: crowdloan
                .target_address
                .map(|a| H256::from_slice(a.as_slice()))
                .unwrap_or_else(H256::zero),
            finalized: crowdloan.finalized,
            contributors_count: crowdloan.contributors_count,
        })
    }

    #[precompile::public("getContribution(uint32,bytes32)")]
    #[precompile::view]
    fn get_contribution(
        _handle: &mut impl PrecompileHandle,
        crowdloan_id: u32,
        coldkey: H256,
    ) -> EvmResult<u64> {
        let coldkey = R::AccountId::from(coldkey.0);
        let contribution = pallet_crowdloan::Contributions::<R>::get(crowdloan_id, coldkey).ok_or(
            PrecompileFailure::Error {
                exit_status: ExitError::Other("Crowdloan or contribution not found".into()),
            },
        )?;

        Ok(contribution)
    }

    #[precompile::public("create(uint64,uint64,uint64,uint32,address)")]
    #[precompile::payable]
    fn create(
        handle: &mut impl PrecompileHandle,
        deposit: u64,
        min_contribution: u64,
        cap: u64,
        end: u32,
        target_address: Address,
    ) -> EvmResult<()> {
        let who = handle.caller_account_id::<R>();
        let target_address = R::AddressMapping::into_account_id(target_address.0);
        let call = pallet_crowdloan::Call::<R>::create {
            deposit,
            min_contribution,
            cap,
            end: end.into(),
            call: None,
            target_address: Some(target_address),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(who))
    }

    #[precompile::public("contribute(uint32,uint64)")]
    #[precompile::payable]
    fn contribute(
        handle: &mut impl PrecompileHandle,
        crowdloan_id: u32,
        amount: u64,
    ) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let call = pallet_crowdloan::Call::<R>::contribute {
            crowdloan_id,
            amount,
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("withdraw(uint32)")]
    #[precompile::payable]
    fn withdraw(handle: &mut impl PrecompileHandle, crowdloan_id: u32) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let call = pallet_crowdloan::Call::<R>::withdraw { crowdloan_id };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("finalize(uint32)")]
    #[precompile::payable]
    fn finalize(handle: &mut impl PrecompileHandle, crowdloan_id: u32) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let call = pallet_crowdloan::Call::<R>::finalize { crowdloan_id };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("refund(uint32)")]
    #[precompile::payable]
    fn refund(handle: &mut impl PrecompileHandle, crowdloan_id: u32) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let call = pallet_crowdloan::Call::<R>::refund { crowdloan_id };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("dissolve(uint32)")]
    #[precompile::payable]
    fn dissolve(handle: &mut impl PrecompileHandle, crowdloan_id: u32) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let call = pallet_crowdloan::Call::<R>::dissolve { crowdloan_id };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("updateMinContribution(uint32,uint64)")]
    #[precompile::payable]
    fn update_min_contribution(
        handle: &mut impl PrecompileHandle,
        crowdloan_id: u32,
        new_min_contribution: u64,
    ) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let call = pallet_crowdloan::Call::<R>::update_min_contribution {
            crowdloan_id,
            new_min_contribution,
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("updateEnd(uint32,uint32)")]
    #[precompile::payable]
    fn update_end(
        handle: &mut impl PrecompileHandle,
        crowdloan_id: u32,
        new_end: u32,
    ) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let call = pallet_crowdloan::Call::<R>::update_end {
            crowdloan_id,
            new_end: new_end.into(),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("updateCap(uint32,uint64)")]
    #[precompile::payable]
    fn update_cap(
        handle: &mut impl PrecompileHandle,
        crowdloan_id: u32,
        new_cap: u64,
    ) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let call = pallet_crowdloan::Call::<R>::update_cap {
            crowdloan_id,
            new_cap,
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }
}

#[derive(Codec)]
struct CrowdloanInfo {
    creator: H256,
    deposit: u64,
    min_contribution: u64,
    end: u32,
    cap: u64,
    funds_account: H256,
    raised: u64,
    has_target_address: bool,
    target_address: H256,
    finalized: bool,
    contributors_count: u32,
}
