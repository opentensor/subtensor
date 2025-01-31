use pallet_evm::{
    BalanceConverter, ExitError, ExitSucceed, PrecompileHandle, PrecompileOutput, PrecompileResult,
};
use sp_runtime::traits::UniqueSaturatedInto;
use sp_std::vec;

use crate::precompiles::{
    contract_to_origin, get_method_id, get_pubkey, get_slice, try_dispatch_runtime_call,
};
use crate::Runtime;

pub const BALANCE_TRANSFER_INDEX: u64 = 2048;
// ss58 public key i.e., the contract sends funds it received to the destination address from the
// method parameter.
const CONTRACT_ADDRESS_SS58: [u8; 32] = [
    0x07, 0xec, 0x71, 0x2a, 0x5d, 0x38, 0x43, 0x4d, 0xdd, 0x03, 0x3f, 0x8f, 0x02, 0x4e, 0xcd, 0xfc,
    0x4b, 0xb5, 0x95, 0x1c, 0x13, 0xc3, 0x08, 0x5c, 0x39, 0x9c, 0x8a, 0x5f, 0x62, 0x93, 0x70, 0x5d,
];

pub struct BalanceTransferPrecompile;

impl BalanceTransferPrecompile {
    pub fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
        let txdata = handle.input();

        // Match method ID: keccak256("transfer(bytes32)")
        let method = get_slice(txdata, 0, 4)?;
        if get_method_id("transfer(bytes32)") != method {
            return Ok(PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                output: vec![],
            });
        }

        // Forward all received value to the destination address
        let amount = handle.context().apparent_value;

        // Use BalanceConverter to convert EVM amount to Substrate balance
        let amount_sub =
            <Runtime as pallet_evm::Config>::BalanceConverter::into_substrate_balance(amount)
                .ok_or(ExitError::OutOfFund)?;

        if amount_sub.is_zero() {
            return Ok(PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                output: vec![],
            });
        }

        let address_bytes_dst = get_slice(txdata, 4, 36)?;
        let (account_id_dst, _) = get_pubkey(address_bytes_dst)?;

        let call = pallet_balances::Call::<Runtime>::transfer_allow_death {
            dest: account_id_dst.into(),
            value: amount_sub.unique_saturated_into(),
        };

        try_dispatch_runtime_call(handle, call, contract_to_origin(&CONTRACT_ADDRESS_SS58)?)
    }
}
