use core::marker::PhantomData;
use sp_core::{
	crypto::ByteArray, hashing::keccak_256, H160
};
use sp_runtime::AccountId32;

use pallet_evm::{
	ExitError, IsPrecompileResult, Precompile, PrecompileHandle, PrecompileResult, PrecompileSet,
	PrecompileFailure
};
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};

// Include custom precompiles
mod balance_transfer;

use balance_transfer::*;

pub struct FrontierPrecompiles<R>(PhantomData<R>);

impl<R> FrontierPrecompiles<R>
where
	R: pallet_evm::Config,
{
	pub fn new() -> Self {
		Self(Default::default())
	}
	pub fn used_addresses() -> [H160; 8] {
		[
			hash(1),
			hash(2),
			hash(3),
			hash(4),
			hash(5),
			hash(1024),
			hash(1025),
			hash(BALANCE_TRANSFER_INDEX)
		]
	}
}
impl<R> PrecompileSet for FrontierPrecompiles<R>
where
	R: pallet_evm::Config,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		match handle.code_address() {
			// Ethereum precompiles :
			a if a == hash(1) => Some(ECRecover::execute(handle)),
			a if a == hash(2) => Some(Sha256::execute(handle)),
			a if a == hash(3) => Some(Ripemd160::execute(handle)),
			a if a == hash(4) => Some(Identity::execute(handle)),
			a if a == hash(5) => Some(Modexp::execute(handle)),
			// Non-Frontier specific nor Ethereum precompiles :
			a if a == hash(1024) => Some(Sha3FIPS256::execute(handle)),
			a if a == hash(1025) => Some(ECRecoverPublicKey::execute(handle)),
			a if a == hash(BALANCE_TRANSFER_INDEX) => Some(BalanceTransferPrecompile::execute(handle)),
			_ => None,
		}
	}

	fn is_precompile(&self, address: H160, _gas: u64) -> IsPrecompileResult {
		IsPrecompileResult::Answer {
			is_precompile: Self::used_addresses().contains(&address),
			extra_cost: 0,
		}
	}
}

fn hash(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}

/// Returns Ethereum method ID from an str method signature
/// 
pub fn get_method_id(method_signature: &str) -> [u8; 4] {
    // Calculate the full Keccak-256 hash of the method signature
    let hash = keccak_256(method_signature.as_bytes());

    // Extract the first 4 bytes to get the method ID
    let method_id = [
        hash[0],
        hash[1],
        hash[2],
        hash[3],
    ];

    method_id
}

/// Convert bytes to AccountId32 with PrecompileFailure as Error
/// which consumes all gas
/// 
pub fn bytes_to_account_id(account_id_bytes: &[u8]) -> Result<AccountId32, PrecompileFailure> {
    AccountId32::from_slice(&account_id_bytes).map_err(|_| {
        log::info!("Error parsing account id bytes {:?}", account_id_bytes);
		PrecompileFailure::Error {
			exit_status: ExitError::InvalidRange
		}
    })
}
