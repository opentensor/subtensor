use core::marker::PhantomData;

use fp_evm::PrecompileHandle;
use precompile_utils::EvmResult;
use sp_core::H256;

use crate::PrecompileExt;

/// Drand precompile for smart contract access to Drand beacon randomness.
///
/// This precompile allows smart contracts to read verifiable randomness from the
/// Drand Quicknet beacon that is bridged on-chain by the Drand pallet.
pub struct DrandPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for DrandPrecompile<R>
where
    R: frame_system::Config + pallet_drand::Config,
    R::AccountId: From<[u8; 32]>,
{
    const INDEX: u64 = 2062;
}

#[precompile_utils::precompile]
impl<R> DrandPrecompile<R>
where
    R: frame_system::Config + pallet_drand::Config,
    R::AccountId: From<[u8; 32]>,
{
    /// Get the 32-byte randomness for a specific Drand round.
    ///
    /// Returns the SHA256 hash of the BLS signature for the given round.
    /// Returns 32 zero bytes if no pulse exists for the round.
    ///
    /// # Arguments
    /// * `round` - The Drand round number (u64)
    ///
    /// # Returns
    /// * `bytes32` - The 32-byte randomness, or zeros if round not stored
    #[precompile::public("getRandomness(uint64)")]
    #[precompile::view]
    fn get_randomness(_: &mut impl PrecompileHandle, round: u64) -> EvmResult<H256> {
        let randomness = pallet_drand::Pallet::<R>::random_at(round);
        Ok(H256::from(randomness))
    }

    /// Get the last Drand round that has been stored on-chain.
    ///
    /// Returns 0 if no pulses have been stored yet.
    ///
    /// # Returns
    /// * `uint64` - The last stored round number
    #[precompile::public("getLastStoredRound()")]
    #[precompile::view]
    fn get_last_stored_round(_: &mut impl PrecompileHandle) -> EvmResult<u64> {
        Ok(pallet_drand::LastStoredRound::<R>::get())
    }
}
