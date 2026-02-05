use core::marker::PhantomData;

use fp_evm::PrecompileHandle;
use precompile_utils::EvmResult;
use sp_core::U256;
use subtensor_runtime_common::{Currency, NetUid};

use crate::PrecompileExt;

/// NeuronRegistrationCost precompile for smart contract access to neuron registration cost.
///
/// This precompile allows smart contracts to query the current burn cost
/// required to register a neuron on a subnet.
pub struct NeuronRegistrationCostPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for NeuronRegistrationCostPrecompile<R>
where
    R: frame_system::Config + pallet_subtensor::Config,
    R::AccountId: From<[u8; 32]>,
{
    // Using 2062 as the next available index after VotingPower (2061)
    const INDEX: u64 = 2062;
}

#[precompile_utils::precompile]
impl<R> NeuronRegistrationCostPrecompile<R>
where
    R: frame_system::Config + pallet_subtensor::Config,
    R::AccountId: From<[u8; 32]>,
{
    /// Get the current burn cost to register a neuron on a subnet.
    ///
    /// This is the amount of TAO (in RAO) that will be burned when registering.
    /// The cost can change dynamically based on network activity.
    ///
    /// # Arguments
    /// * `netuid` - The subnet identifier (u16)
    ///
    /// # Returns
    /// * `u256` - The burn cost in RAO (1 TAO = 10^9 RAO)
    #[precompile::public("getBurn(uint16)")]
    #[precompile::view]
    fn get_burn(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<U256> {
        let burn = pallet_subtensor::Pallet::<R>::get_burn(NetUid::from(netuid));
        Ok(U256::from(burn.to_u64()))
    }

    /// Get the minimum burn cost for a subnet.
    ///
    /// # Arguments
    /// * `netuid` - The subnet identifier (u16)
    ///
    /// # Returns
    /// * `u256` - The minimum burn cost in RAO
    #[precompile::public("getMinBurn(uint16)")]
    #[precompile::view]
    fn get_min_burn(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<U256> {
        let min_burn = pallet_subtensor::Pallet::<R>::get_min_burn(NetUid::from(netuid));
        Ok(U256::from(min_burn.to_u64()))
    }

    /// Get the maximum burn cost for a subnet.
    ///
    /// # Arguments
    /// * `netuid` - The subnet identifier (u16)
    ///
    /// # Returns
    /// * `u256` - The maximum burn cost in RAO
    #[precompile::public("getMaxBurn(uint16)")]
    #[precompile::view]
    fn get_max_burn(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<U256> {
        let max_burn = pallet_subtensor::Pallet::<R>::get_max_burn(NetUid::from(netuid));
        Ok(U256::from(max_burn.to_u64()))
    }

    /// Check if registration is allowed on a subnet.
    ///
    /// # Arguments
    /// * `netuid` - The subnet identifier (u16)
    ///
    /// # Returns
    /// * `bool` - True if registration is allowed
    #[precompile::public("isRegistrationAllowed(uint16)")]
    #[precompile::view]
    fn is_registration_allowed(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<bool> {
        Ok(pallet_subtensor::Pallet::<R>::get_network_registration_allowed(NetUid::from(netuid)))
    }
}
