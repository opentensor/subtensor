use frame_system::RawOrigin;
use pallet_evm::{
    AddressMapping, ExitError, HashedAddressMapping, PrecompileFailure, PrecompileHandle,
    PrecompileResult,
};
use precompile_utils::EvmResult;
use sp_core::H256;
use sp_runtime::traits::BlakeTwo256;
use sp_runtime::AccountId32;
use sp_std::vec;
use sp_std::vec::Vec;

use crate::precompiles::{
    get_method_id, parse_netuid, parse_pubkey, parse_slice, PrecompileExt, PrecompileHandleExt,
};
use crate::{Runtime, RuntimeCall};

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

    #[precompile::public("commitWeights(uint16,uint256)")]
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
}
