/// The blake2_128 precompile.
use fp_evm::{ExitSucceed, LinearCostPrecompile, PrecompileFailure};
use sp_std::vec::Vec;
pub struct BlakeTwo128;

impl LinearCostPrecompile for BlakeTwo128 {
    const BASE: u64 = 60;
    const WORD: u64 = 12;

    fn execute(input: &[u8], _cost: u64) -> Result<(ExitSucceed, Vec<u8>), PrecompileFailure> {
        let ret = sp_io::hashing::blake2_128(input);
        Ok((ExitSucceed::Returned, ret.to_vec()))
    }
}

pub struct BlakeTwo256;

impl LinearCostPrecompile for BlakeTwo256 {
    const BASE: u64 = 60;
    const WORD: u64 = 12;

    fn execute(input: &[u8], _cost: u64) -> Result<(ExitSucceed, Vec<u8>), PrecompileFailure> {
        let ret = sp_io::hashing::blake2_256(input);
        Ok((ExitSucceed::Returned, ret.to_vec()))
    }
}
