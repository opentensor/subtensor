use core::marker::PhantomData;

use pallet_evm::PrecompileHandle;
use precompile_utils::{EvmResult, prelude::UnboundedBytes};
use sp_core::H256;

use crate::PrecompileExt;

pub(crate) struct DrandPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for DrandPrecompile<R>
where
    R: frame_system::Config + pallet_drand::Config + pallet_evm::Config,
    R::AccountId: From<[u8; 32]>,
{
    const INDEX: u64 = 2065;
}

#[precompile_utils::precompile]
impl<R> DrandPrecompile<R>
where
    R: frame_system::Config + pallet_drand::Config + pallet_evm::Config,
{
    /// Returns the last stored drand round number.
    #[precompile::public("getLastStoredRound()")]
    #[precompile::view]
    fn get_last_stored_round(_handle: &mut impl PrecompileHandle) -> EvmResult<u64> {
        Ok(pallet_drand::LastStoredRound::<R>::get())
    }

    /// Returns the oldest stored drand round number.
    #[precompile::public("getOldestStoredRound()")]
    #[precompile::view]
    fn get_oldest_stored_round(_handle: &mut impl PrecompileHandle) -> EvmResult<u64> {
        Ok(pallet_drand::OldestStoredRound::<R>::get())
    }

    /// Returns the pulse (randomness, signature) for a specific round.
    /// If the round is not found, returns empty bytes.
    #[precompile::public("getPulse(uint64)")]
    #[precompile::view]
    fn get_pulse(
        _handle: &mut impl PrecompileHandle,
        round: u64,
    ) -> EvmResult<(UnboundedBytes, UnboundedBytes)> {
        match pallet_drand::Pulses::<R>::get(round) {
            Some(pulse) => {
                let randomness: UnboundedBytes = pulse.randomness.into_inner().into();
                let signature: UnboundedBytes = pulse.signature.into_inner().into();
                Ok((randomness, signature))
            }
            None => Ok((
                UnboundedBytes::from(&b""[..]),
                UnboundedBytes::from(&b""[..]),
            )),
        }
    }

    /// Returns the randomness from the latest stored round as bytes32.
    /// Returns zero bytes if no pulse is stored.
    #[precompile::public("getCurrentRandomness()")]
    #[precompile::view]
    fn get_current_randomness(_handle: &mut impl PrecompileHandle) -> EvmResult<H256> {
        let last_round = pallet_drand::LastStoredRound::<R>::get();
        match pallet_drand::Pulses::<R>::get(last_round) {
            Some(pulse) => {
                let rand = pulse.randomness.into_inner();
                let bounded: [u8; 32] = rand.try_into().unwrap_or([0u8; 32]);
                Ok(H256::from(bounded))
            }
            None => Ok(H256::zero()),
        }
    }
}
