use frame_system::RawOrigin;
use pallet_evm::PrecompileHandle;
use precompile_utils::{EvmResult, prelude::UnboundedBytes};
use sp_core::H256;
use sp_std::vec::Vec;

use crate::Runtime;
use crate::precompiles::{PrecompileExt, PrecompileHandleExt, parse_pubkey};

pub struct NeuronPrecompile;

impl PrecompileExt for NeuronPrecompile {
    const INDEX: u64 = 2052;
    const ADDRESS_SS58: [u8; 32] = [
        0xbc, 0x46, 0x35, 0x79, 0xbc, 0x99, 0xf9, 0xee, 0x7c, 0x59, 0xed, 0xee, 0x20, 0x61, 0xa3,
        0x09, 0xd2, 0x1e, 0x68, 0xd5, 0x39, 0xb6, 0x40, 0xec, 0x66, 0x46, 0x90, 0x30, 0xab, 0x74,
        0xc1, 0xdb,
    ];
}

#[precompile_utils::precompile]
impl NeuronPrecompile {
    #[precompile::public("setWeights(uint16,uint16[],uint16[],uint64)")]
    #[precompile::payable]
    pub fn set_weights(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        dests: Vec<u16>,
        weights: Vec<u16>,
        version_key: u64,
    ) -> EvmResult<()> {
        let call = pallet_subtensor::Call::<Runtime>::set_weights {
            netuid,
            dests,
            weights,
            version_key,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("commitWeights(uint16,bytes32)")]
    #[precompile::payable]
    pub fn commit_weights(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        commit_hash: H256,
    ) -> EvmResult<()> {
        let call = pallet_subtensor::Call::<Runtime>::commit_weights {
            netuid,
            commit_hash,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("revealWeights(uint16,uint16[],uint16[],uint16[],uint64)")]
    #[precompile::payable]
    pub fn reveal_weights(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        uids: Vec<u16>,
        values: Vec<u16>,
        salt: Vec<u16>,
        version_key: u64,
    ) -> EvmResult<()> {
        let call = pallet_subtensor::Call::<Runtime>::reveal_weights {
            netuid,
            uids,
            values,
            salt,
            version_key,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("burnedRegister(uint16,bytes32)")]
    #[precompile::payable]
    fn burned_register(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        hotkey: H256,
    ) -> EvmResult<()> {
        let coldkey = handle.caller_account_id();
        let (hotkey, _) = parse_pubkey(hotkey.as_bytes())?;
        let call = pallet_subtensor::Call::<Runtime>::burned_register { netuid, hotkey };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(coldkey))
    }

    #[precompile::public("serveAxon(uint16,uint32,uint128,uint16,uint8,uint8,uint8,uint8)")]
    #[precompile::payable]
    #[allow(clippy::too_many_arguments)]
    fn serve_axon(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        version: u32,
        ip: u128,
        port: u16,
        ip_type: u8,
        protocol: u8,
        placeholder1: u8,
        placeholder2: u8,
    ) -> EvmResult<()> {
        let call = pallet_subtensor::Call::<Runtime>::serve_axon {
            netuid,
            version,
            ip,
            port,
            ip_type,
            protocol,
            placeholder1,
            placeholder2,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public(
        "serveAxonTls(uint16,uint32,uint128,uint16,uint8,uint8,uint8,uint8,bytes)"
    )]
    #[precompile::payable]
    #[allow(clippy::too_many_arguments)]
    fn serve_axon_tls(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        version: u32,
        ip: u128,
        port: u16,
        ip_type: u8,
        protocol: u8,
        placeholder1: u8,
        placeholder2: u8,
        certificate: UnboundedBytes,
    ) -> EvmResult<()> {
        let call = pallet_subtensor::Call::<Runtime>::serve_axon_tls {
            netuid,
            version,
            ip,
            port,
            ip_type,
            protocol,
            placeholder1,
            placeholder2,
            certificate: certificate.into(),
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }

    #[precompile::public("servePrometheus(uint16,uint32,uint128,uint16,uint8)")]
    #[precompile::payable]
    #[allow(clippy::too_many_arguments)]
    fn serve_prometheus(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        version: u32,
        ip: u128,
        port: u16,
        ip_type: u8,
    ) -> EvmResult<()> {
        let call = pallet_subtensor::Call::<Runtime>::serve_prometheus {
            netuid,
            version,
            ip,
            port,
            ip_type,
        };

        handle.try_dispatch_runtime_call(call, RawOrigin::Signed(handle.caller_account_id()))
    }
}
