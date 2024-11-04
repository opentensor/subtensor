extern crate alloc;

use alloc::vec::Vec;

use crate::precompiles::get_slice;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use fp_evm::{ExitError, ExitSucceed, LinearCostPrecompile, PrecompileFailure};

pub const EDVERIFY_PRECOMPILE_INDEX: u64 = 1026;

pub struct Ed25519Verify;

impl LinearCostPrecompile for Ed25519Verify {
    const BASE: u64 = 15;
    const WORD: u64 = 3;

    fn execute(input: &[u8], _: u64) -> Result<(ExitSucceed, Vec<u8>), PrecompileFailure> {
        if input.len() < 132 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("input must contain 128 bytes".into()),
            });
        };

        let mut buf = [0u8; 32];

        let msg = get_slice(input, 4, 36)?;
        let pk = VerifyingKey::try_from(get_slice(input, 36, 68)?).map_err(|_| {
            PrecompileFailure::Error {
                exit_status: ExitError::Other("Public key recover failed".into()),
            }
        })?;
        let sig = Signature::try_from(get_slice(input, 68, 132)?).map_err(|_| {
            PrecompileFailure::Error {
                exit_status: ExitError::Other("Signature recover failed".into()),
            }
        })?;

        if pk.verify(msg, &sig).is_ok() {
            buf[31] = 1u8;
        };

        Ok((ExitSucceed::Returned, buf.to_vec()))
    }
}
