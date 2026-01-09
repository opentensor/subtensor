use alloc::{boxed::Box, string::String};
use core::marker::PhantomData;

use fp_evm::{ExitError, PrecompileFailure};
use frame_support::dispatch::{DispatchInfo, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::IsSubType;
use frame_system::RawOrigin;
use pallet_evm::AddressMapping;
use pallet_evm::PrecompileHandle;
use precompile_utils::{EvmResult, solidity::Codec};
use sp_core::{ByteArray, H256};
use sp_runtime::{
    Percent,
    traits::{AsSystemOriginSigner, Dispatchable, UniqueSaturatedInto},
};
use subtensor_runtime_common::NetUid;

use crate::{PrecompileExt, PrecompileHandleExt};

pub struct LeasingPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for LeasingPrecompile<R>
where
    R: frame_system::Config
        + pallet_balances::Config
        + pallet_evm::Config
        + pallet_subtensor::Config
        + pallet_crowdloan::Config
        + pallet_shield::Config
        + Send
        + Sync
        + scale_info::TypeInfo,
    R::AccountId: From<[u8; 32]> + ByteArray,
    <R as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<R::AccountId> + Clone,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_crowdloan::Call<R>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<pallet_balances::Call<R>>
        + IsSubType<pallet_subtensor::Call<R>>
        + IsSubType<pallet_shield::Call<R>>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
{
    const INDEX: u64 = 2058;
}

#[precompile_utils::precompile]
impl<R> LeasingPrecompile<R>
where
    R: frame_system::Config
        + pallet_balances::Config
        + pallet_evm::Config
        + pallet_subtensor::Config
        + pallet_crowdloan::Config
        + pallet_shield::Config
        + Send
        + Sync
        + scale_info::TypeInfo,
    R::AccountId: From<[u8; 32]> + ByteArray,
    <R as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<R::AccountId> + Clone,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_crowdloan::Call<R>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<pallet_balances::Call<R>>
        + IsSubType<pallet_subtensor::Call<R>>
        + IsSubType<pallet_shield::Call<R>>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
{
    #[precompile::public("getLease(uint32)")]
    #[precompile::view]
    fn get_lease(_handle: &mut impl PrecompileHandle, lease_id: u32) -> EvmResult<LeaseInfo> {
        let lease =
            pallet_subtensor::SubnetLeases::<R>::get(lease_id).ok_or(PrecompileFailure::Error {
                exit_status: ExitError::Other("Lease not found".into()),
            })?;

        Ok(LeaseInfo {
            beneficiary: H256::from_slice(lease.beneficiary.as_slice()),
            coldkey: H256::from_slice(lease.coldkey.as_slice()),
            hotkey: H256::from_slice(lease.hotkey.as_slice()),
            emissions_share: lease.emissions_share.deconstruct(),
            has_end_block: lease.end_block.is_some(),
            end_block: lease
                .end_block
                .map(|b| b.unique_saturated_into())
                .unwrap_or(0),
            netuid: lease.netuid.into(),
            cost: lease.cost,
        })
    }

    #[precompile::public("getContributorShare(uint32,bytes32)")]
    #[precompile::view]
    fn get_contributor_share(
        _handle: &mut impl PrecompileHandle,
        lease_id: u32,
        contributor: H256,
    ) -> EvmResult<(u128, u128)> {
        let contributor = R::AccountId::from(contributor.0);
        let share = pallet_subtensor::SubnetLeaseShares::<R>::get(lease_id, contributor);

        Ok((share.int().to_bits(), share.frac().to_bits()))
    }

    #[precompile::public("getLeaseIdForSubnet(uint16)")]
    #[precompile::view]
    fn get_lease_id_for_subnet(_handle: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u32> {
        let lease_id = pallet_subtensor::SubnetUidToLeaseId::<R>::get(NetUid::from(netuid)).ok_or(
            PrecompileFailure::Error {
                exit_status: ExitError::Other("Lease not found for netuid".into()),
            },
        )?;

        Ok(lease_id.into())
    }

    #[precompile::public("createLeaseCrowdloan(uint64,uint64,uint64,uint32,uint8,bool,uint32)")]
    #[precompile::payable]
    #[allow(clippy::too_many_arguments)]
    fn create_lease_crowdloan(
        handle: &mut impl PrecompileHandle,
        crowdloan_deposit: u64,
        crowdloan_min_contribution: u64,
        crowdloan_cap: u64,
        crowdloan_end: u32,
        leasing_emissions_share: u8,
        has_leasing_end_block: bool,
        leasing_end_block: u32,
    ) -> EvmResult<()> {
        let who = handle.caller_account_id::<R>();

        let leasing_end_block = if has_leasing_end_block {
            Some(leasing_end_block.into())
        } else {
            None
        };

        let leasing_call = {
            let call = pallet_subtensor::Call::<R>::register_leased_network {
                emissions_share: Percent::from_percent(leasing_emissions_share),
                end_block: leasing_end_block,
            };
            let system_call: <R as frame_system::Config>::RuntimeCall = call.into();
            Box::new(system_call.into())
        };

        let crowdloan_call = pallet_crowdloan::Call::<R>::create {
            deposit: crowdloan_deposit,
            min_contribution: crowdloan_min_contribution,
            cap: crowdloan_cap,
            end: crowdloan_end.into(),
            call: Some(leasing_call),
            target_address: None,
        };

        handle.try_dispatch_runtime_call::<R, _>(crowdloan_call, RawOrigin::Signed(who))
    }

    #[precompile::public("terminateLease(uint32,bytes32)")]
    #[precompile::payable]
    fn terminate_lease(
        handle: &mut impl PrecompileHandle,
        lease_id: u32,
        hotkey: H256,
    ) -> EvmResult<()> {
        let who = handle.caller_account_id::<R>();
        let hotkey = R::AccountId::from(hotkey.0);
        let call = pallet_subtensor::Call::<R>::terminate_lease { lease_id, hotkey };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(who))
    }
}

#[derive(Codec)]
struct LeaseInfo {
    beneficiary: H256,
    coldkey: H256,
    hotkey: H256,
    emissions_share: u8,
    has_end_block: bool,
    end_block: u32,
    netuid: u16,
    cost: u64,
}
