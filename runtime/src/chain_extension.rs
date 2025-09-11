use codec::Encode;
use pallet_contracts::chain_extension::{
    ChainExtension, Environment, Ext, InitState, RetVal, SysConfig,
};
use sp_runtime::{AccountId32, DispatchError};
use subtensor_runtime_common::NetUid;

use crate::{Runtime, SubtensorModule};

#[derive(Default)]
pub struct SubtensorChainExtension;

impl ChainExtension<Runtime> for SubtensorChainExtension {
    fn call<E: Ext>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
    where
        E::T: SysConfig,
    {
        let func_id = env.func_id();

        match func_id {
            // Function ID 1001: get_stake_info_for_hotkey_coldkey_netuid
            1001 => {
                let mut env = env.buf_in_buf_out();

                let input: (AccountId32, AccountId32, NetUid) = env
                    .read_as()
                    .map_err(|_| DispatchError::Other("Failed to decode input parameters"))?;

                let (hotkey, coldkey, netuid) = input;

                let stake_info = SubtensorModule::get_stake_info_for_hotkey_coldkey_netuid(
                    hotkey, coldkey, netuid,
                );

                let encoded_result = stake_info.encode();

                env.write(&encoded_result, false, None)
                    .map_err(|_| DispatchError::Other("Failed to write output"))?;

                Ok(RetVal::Converging(0))
            }
            _ => {
                log::error!("Called an unregistered chain extension function: {func_id}",);
                Err(DispatchError::Other("Unimplemented function ID"))
            }
        }
    }

    fn enabled() -> bool {
        true
    }
}
