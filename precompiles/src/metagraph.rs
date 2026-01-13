use alloc::string::String;
use core::marker::PhantomData;

use fp_evm::{ExitError, PrecompileFailure, PrecompileHandle};
use pallet_rate_limiting::RateLimitingInterface;
use pallet_subtensor::AxonInfo as SubtensorModuleAxonInfo;
use precompile_utils::{EvmResult, solidity::Codec};
use sp_core::{ByteArray, H256};
use sp_runtime::SaturatedConversion;
use subtensor_runtime_common::{Currency, NetUid, rate_limiting};

use crate::PrecompileExt;

pub struct MetagraphPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for MetagraphPrecompile<R>
where
    R: frame_system::Config + pallet_subtensor::Config,
    R::AccountId: From<[u8; 32]> + ByteArray,
{
    const INDEX: u64 = 2050;
}

#[precompile_utils::precompile]
impl<R> MetagraphPrecompile<R>
where
    R: frame_system::Config + pallet_subtensor::Config,
    R::AccountId: ByteArray,
{
    #[precompile::public("getUidCount(uint16)")]
    #[precompile::view]
    fn get_uid_count(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::SubnetworkN::<R>::get(NetUid::from(
            netuid,
        )))
    }

    #[precompile::public("getStake(uint16,uint16)")]
    #[precompile::view]
    fn get_stake(_: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<u64> {
        let hotkey = pallet_subtensor::Pallet::<R>::get_hotkey_for_net_and_uid(netuid.into(), uid)
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            })?;

        Ok(pallet_subtensor::Pallet::<R>::get_total_stake_for_hotkey(&hotkey).to_u64())
    }

    #[precompile::public("getRank(uint16,uint16)")]
    #[precompile::view]
    fn get_rank(_: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::Pallet::<R>::get_rank_for_uid(
            netuid.into(),
            uid,
        ))
    }

    #[precompile::public("getTrust(uint16,uint16)")]
    #[precompile::view]
    fn get_trust(_: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::Pallet::<R>::get_trust_for_uid(
            netuid.into(),
            uid,
        ))
    }

    #[precompile::public("getConsensus(uint16,uint16)")]
    #[precompile::view]
    fn get_consensus(_: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::Pallet::<R>::get_consensus_for_uid(
            netuid.into(),
            uid,
        ))
    }

    #[precompile::public("getIncentive(uint16,uint16)")]
    #[precompile::view]
    fn get_incentive(_: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::Pallet::<R>::get_incentive_for_uid(
            netuid.into(),
            uid,
        ))
    }

    #[precompile::public("getDividends(uint16,uint16)")]
    #[precompile::view]
    fn get_dividends(_: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::Pallet::<R>::get_dividends_for_uid(
            netuid.into(),
            uid,
        ))
    }

    #[precompile::public("getEmission(uint16,uint16)")]
    #[precompile::view]
    fn get_emission(_: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::Pallet::<R>::get_emission_for_uid(netuid.into(), uid).into())
    }

    #[precompile::public("getVtrust(uint16,uint16)")]
    #[precompile::view]
    fn get_vtrust(_: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::Pallet::<R>::get_validator_trust_for_uid(
            netuid.into(),
            uid,
        ))
    }

    #[precompile::public("getValidatorStatus(uint16,uint16)")]
    #[precompile::view]
    fn get_validator_status(
        _: &mut impl PrecompileHandle,
        netuid: u16,
        uid: u16,
    ) -> EvmResult<bool> {
        Ok(pallet_subtensor::Pallet::<R>::get_validator_permit_for_uid(
            netuid.into(),
            uid,
        ))
    }

    #[precompile::public("getLastUpdate(uint16,uint16)")]
    #[precompile::view]
    fn get_last_update(_: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<u64> {
        let usage = rate_limiting::RateLimitUsageKey::<R::AccountId>::SubnetNeuron {
            netuid: netuid.into(),
            uid,
        };
        let block = <R as pallet_subtensor::Config>::RateLimiting::last_seen(
            rate_limiting::GROUP_WEIGHTS_SUBNET,
            Some(usage),
        )
        .map(|block| block.saturated_into::<u64>())
        .unwrap_or(0);

        Ok(block)
    }

    #[precompile::public("getIsActive(uint16,uint16)")]
    #[precompile::view]
    fn get_is_active(_: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<bool> {
        Ok(pallet_subtensor::Pallet::<R>::get_active_for_uid(
            netuid.into(),
            uid,
        ))
    }

    #[precompile::public("getAxon(uint16,uint16)")]
    #[precompile::view]
    fn get_axon(_: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<AxonInfo> {
        let hotkey = pallet_subtensor::Pallet::<R>::get_hotkey_for_net_and_uid(netuid.into(), uid)
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("hotkey not found".into()),
            })?;

        Ok(pallet_subtensor::Pallet::<R>::get_axon_info(netuid.into(), &hotkey).into())
    }

    #[precompile::public("getHotkey(uint16,uint16)")]
    #[precompile::view]
    fn get_hotkey(_: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<H256> {
        pallet_subtensor::Pallet::<R>::get_hotkey_for_net_and_uid(netuid.into(), uid)
            .map(|acc| H256::from_slice(acc.as_slice()))
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            })
    }

    #[precompile::public("getColdkey(uint16,uint16)")]
    #[precompile::view]
    fn get_coldkey(_: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<H256> {
        let hotkey = pallet_subtensor::Pallet::<R>::get_hotkey_for_net_and_uid(netuid.into(), uid)
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            })?;
        let coldkey = pallet_subtensor::Owner::<R>::get(&hotkey);

        Ok(H256::from_slice(coldkey.as_slice()))
    }
}

#[derive(Codec)]
struct AxonInfo {
    block: u64,
    version: u32,
    ip: u128,
    port: u16,
    ip_type: u8,
    protocol: u8,
}

impl From<SubtensorModuleAxonInfo> for AxonInfo {
    fn from(value: SubtensorModuleAxonInfo) -> Self {
        Self {
            block: value.block,
            version: value.version,
            ip: value.ip,
            port: value.port,
            ip_type: value.ip_type,
            protocol: value.protocol,
        }
    }
}
