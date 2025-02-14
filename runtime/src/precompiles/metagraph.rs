extern crate alloc;
use alloc::string::String;

use fp_evm::{
    ExitError, ExitSucceed, PrecompileFailure, PrecompileHandle, PrecompileOutput, PrecompileResult,
};
use pallet_subtensor::AxonInfo as SubtensorModuleAxonInfo;
use precompile_utils::{solidity::Codec, EvmResult};
use sp_core::{ByteArray, H256, U256};
use sp_std::vec;

use crate::precompiles::{get_method_id, parse_slice, PrecompileExt, PrecompileHandleExt};
use crate::Runtime;

pub struct MetagraphPrecompile;

#[precompile_utils::precompile]
impl MetagraphPrecompile {
    #[precompile::public("getUidCount(uint16)")]
    #[precompile::view]
    fn get_uid_count(handle: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::SubnetworkN::<Runtime>::get(netuid))
    }

    #[precompile::public("getStake(uint16,uint16)")]
    #[precompile::view]
    fn get_stake(handle: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<u64> {
        let hotkey = pallet_subtensor::Pallet::<Runtime>::get_hotkey_for_net_and_uid(netuid, uid)
            .map_err(|_| PrecompileFailure::Error {
            exit_status: ExitError::InvalidRange,
        })?;

        Ok(pallet_subtensor::Pallet::<Runtime>::get_total_stake_for_hotkey(&hotkey))
    }

    #[precompile::public("getRank(uint16,uint16)")]
    #[precompile::view]
    fn get_rank(handle: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::Pallet::<Runtime>::get_rank_for_uid(
            netuid, uid,
        ))
    }

    #[precompile::public("getTrust(uint16,uint16)")]
    #[precompile::view]
    fn get_trust(handle: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::Pallet::<Runtime>::get_trust_for_uid(
            netuid, uid,
        ))
    }

    #[precompile::public("getConsensus(uint16,uint16)")]
    #[precompile::view]
    fn get_consensus(handle: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::Pallet::<Runtime>::get_consensus_for_uid(
            netuid, uid,
        ))
    }

    #[precompile::public("getIncentive(uint16,uint16)")]
    #[precompile::view]
    fn get_incentive(handle: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::Pallet::<Runtime>::get_incentive_for_uid(
            netuid, uid,
        ))
    }

    #[precompile::public("getDividends(uint16,uint16)")]
    #[precompile::view]
    fn get_dividends(handle: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::Pallet::<Runtime>::get_dividends_for_uid(
            netuid, uid,
        ))
    }

    #[precompile::public("getEmission(uint16,uint16)")]
    #[precompile::view]
    fn get_emission(handle: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::Pallet::<Runtime>::get_emission_for_uid(
            netuid, uid,
        ))
    }

    #[precompile::public("getVtrust(uint16,uint16)")]
    #[precompile::view]
    fn get_vtrust(handle: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<u16> {
        Ok(pallet_subtensor::Pallet::<Runtime>::get_validator_trust_for_uid(netuid, uid))
    }

    #[precompile::public("getValidatorStatus(uint16,uint16)")]
    #[precompile::view]
    fn get_validator_status(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        uid: u16,
    ) -> EvmResult<bool> {
        Ok(pallet_subtensor::Pallet::<Runtime>::get_validator_permit_for_uid(netuid, uid))
    }

    #[precompile::public("getLastUpdate(uint16,uint16)")]
    #[precompile::view]
    fn get_last_update(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        uid: u16,
    ) -> EvmResult<u64> {
        Ok(pallet_subtensor::Pallet::<Runtime>::get_last_update_for_uid(netuid, uid))
    }

    #[precompile::public("getIsActive(uint16,uint16)")]
    #[precompile::view]
    fn get_is_active(handle: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<bool> {
        Ok(pallet_subtensor::Pallet::<Runtime>::get_active_for_uid(
            netuid, uid,
        ))
    }

    #[precompile::public("getAxon(uint16,uint16)")]
    #[precompile::view]
    fn get_axon(handle: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<AxonInfo> {
        let hotkey = pallet_subtensor::Pallet::<Runtime>::get_hotkey_for_net_and_uid(netuid, uid)
            .map_err(|_| PrecompileFailure::Error {
            exit_status: ExitError::Other("hotkey not found".into()),
        })?;

        Ok(pallet_subtensor::Pallet::<Runtime>::get_axon_info(netuid, &hotkey).into())
    }

    #[precompile::public("getHotkey(uint16,uint16)")]
    #[precompile::view]
    fn get_hotkey(handle: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<H256> {
        pallet_subtensor::Pallet::<Runtime>::get_hotkey_for_net_and_uid(netuid, uid)
            .map(|acc| H256::from_slice(acc.as_slice()))
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            })
    }

    #[precompile::public("getColdkey(uint16,uint16)")]
    #[precompile::view]
    fn get_coldkey(handle: &mut impl PrecompileHandle, netuid: u16, uid: u16) -> EvmResult<H256> {
        let hotkey = pallet_subtensor::Pallet::<Runtime>::get_hotkey_for_net_and_uid(netuid, uid)
            .map_err(|_| PrecompileFailure::Error {
            exit_status: ExitError::InvalidRange,
        })?;
        let coldkey = pallet_subtensor::Owner::<Runtime>::get(&hotkey);

        Ok(H256::from_slice(coldkey.as_slice()))
    }
}

impl PrecompileExt for MetagraphPrecompile {
    const INDEX: u64 = 2050;
    const ADDRESS_SS58: [u8; 32] = [0; 32];
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
