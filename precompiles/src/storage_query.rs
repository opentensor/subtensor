use core::marker::PhantomData;

use fp_evm::{ExitError, PrecompileFailure};
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use pallet_evm::{ExitSucceed, Precompile, PrecompileHandle, PrecompileOutput};
use precompile_utils::prelude::PrecompileHandleExt;
use sp_runtime::traits::{Dispatchable, StaticLookup};
use sp_std::vec::Vec;

// twox_128("SubtensorModule") which is the pallet prefix
const SUBTENSOR_PREFIX: [u8; 16] = [
    0x65, 0x8f, 0xaa, 0x38, 0x50, 0x70, 0xe0, 0x74, 0xc8, 0x5b, 0xf6, 0xb5, 0x68, 0xcf, 0x05, 0x55,
];

use crate::PrecompileExt;

pub(crate) struct StorageQueryPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for StorageQueryPrecompile<R>
where
    R: frame_system::Config + pallet_subtensor::Config + pallet_evm::Config,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeCall:
        GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    const INDEX: u64 = 2055;
}

impl<R> Precompile for StorageQueryPrecompile<R>
where
    R: frame_system::Config + pallet_subtensor::Config + pallet_evm::Config,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeCall:
        GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    fn execute(handle: &mut impl PrecompileHandle) -> fp_evm::PrecompileResult {
        let input = handle.input();

        if input.len() < SUBTENSOR_PREFIX.len() || !input.starts_with(&SUBTENSOR_PREFIX) {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("Invalid key".into()),
            });
        }

        match sp_io::storage::get(input) {
            Some(value) => {
                let result = value.to_vec();
                handle.record_db_read::<R>(result.len())?;

                Ok(PrecompileOutput {
                    exit_status: ExitSucceed::Returned,
                    output: result,
                })
            }
            None => {
                // We still record a read of 1 byte to avoid spamming.
                handle.record_db_read::<R>(1)?;

                Ok(PrecompileOutput {
                    exit_status: ExitSucceed::Returned,
                    output: Vec::new(),
                })
            }
        }
    }
}
