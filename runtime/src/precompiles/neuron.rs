use frame_system::RawOrigin;
use pallet_evm::{AddressMapping, HashedAddressMapping, PrecompileHandle};
use precompile_utils::EvmResult;
use sp_core::H256;
use sp_runtime::traits::BlakeTwo256;
use sp_runtime::AccountId32;

use crate::precompiles::{get_pubkey, try_dispatch_runtime_call};
use crate::{Runtime, RuntimeCall};

pub const NEURON_PRECOMPILE_INDEX: u64 = 2052;

pub struct NeuronPrecompile;

#[precompile_utils::precompile]
impl NeuronPrecompile {
    #[precompile::public("burnedRegister(uint16,bytes32)")]
    #[precompile::payable]
    fn burned_register(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        hotkey: H256,
    ) -> EvmResult<()> {
        let coldkey =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );
        let (hotkey, _) = get_pubkey(hotkey.as_bytes())?;
        let call =
            RuntimeCall::SubtensorModule(pallet_subtensor::Call::<Runtime>::burned_register {
                netuid,
                hotkey,
            });

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(coldkey))?;

        Ok(())
    }
}
