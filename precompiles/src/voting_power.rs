use core::marker::PhantomData;

use fp_evm::PrecompileHandle;
use precompile_utils::EvmResult;
use sp_core::{ByteArray, H256, U256};
use subtensor_runtime_common::NetUid;

use crate::PrecompileExt;

/// VotingPower precompile for smart contract access to validator voting power.
///
/// This precompile allows smart contracts to query voting power for validators,
/// enabling on-chain governance decisions like slashing and spending.
pub struct VotingPowerPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for VotingPowerPrecompile<R>
where
    R: frame_system::Config + pallet_subtensor::Config,
    R::AccountId: From<[u8; 32]> + ByteArray,
{
    const INDEX: u64 = 2054;
}

#[precompile_utils::precompile]
impl<R> VotingPowerPrecompile<R>
where
    R: frame_system::Config + pallet_subtensor::Config,
    R::AccountId: From<[u8; 32]> + ByteArray,
{
    /// Get voting power for a hotkey on a subnet.
    ///
    /// Returns the EMA of stake for the hotkey, which represents its voting power.
    /// Returns 0 if:
    /// - The hotkey has no voting power entry
    /// - Voting power tracking is not enabled for the subnet
    /// - The hotkey is not registered on the subnet
    ///
    /// # Arguments
    /// * `netuid` - The subnet identifier (u16)
    /// * `hotkey` - The hotkey account ID (bytes32)
    ///
    /// # Returns
    /// * `u256` - The voting power value (in RAO, same precision as stake)
    #[precompile::public("getVotingPower(uint16,bytes32)")]
    #[precompile::view]
    fn get_voting_power(
        _: &mut impl PrecompileHandle,
        netuid: u16,
        hotkey: H256,
    ) -> EvmResult<U256> {
        let hotkey = R::AccountId::from(hotkey.0);
        let voting_power = pallet_subtensor::VotingPower::<R>::get(NetUid::from(netuid), &hotkey);
        Ok(U256::from(voting_power))
    }

    /// Check if voting power tracking is enabled for a subnet.
    ///
    /// # Arguments
    /// * `netuid` - The subnet identifier (u16)
    ///
    /// # Returns
    /// * `bool` - True if voting power tracking is enabled
    #[precompile::public("isVotingPowerTrackingEnabled(uint16)")]
    #[precompile::view]
    fn is_voting_power_tracking_enabled(
        _: &mut impl PrecompileHandle,
        netuid: u16,
    ) -> EvmResult<bool> {
        Ok(pallet_subtensor::VotingPowerTrackingEnabled::<R>::get(
            NetUid::from(netuid),
        ))
    }

    /// Get the block at which voting power tracking will be disabled.
    ///
    /// Returns 0 if not scheduled for disabling.
    /// When non-zero, tracking continues until this block, then stops.
    ///
    /// # Arguments
    /// * `netuid` - The subnet identifier (u16)
    ///
    /// # Returns
    /// * `u64` - The block number at which tracking will be disabled (0 if not scheduled)
    #[precompile::public("getVotingPowerDisableAtBlock(uint16)")]
    #[precompile::view]
    fn get_voting_power_disable_at_block(
        _: &mut impl PrecompileHandle,
        netuid: u16,
    ) -> EvmResult<u64> {
        Ok(pallet_subtensor::VotingPowerDisableAtBlock::<R>::get(
            NetUid::from(netuid),
        ))
    }

    /// Get the EMA alpha value for voting power calculation on a subnet.
    ///
    /// Alpha is stored with 18 decimal precision (1.0 = 10^18).
    /// Higher alpha = faster response to stake changes.
    ///
    /// # Arguments
    /// * `netuid` - The subnet identifier (u16)
    ///
    /// # Returns
    /// * `u64` - The alpha value (with 18 decimal precision)
    #[precompile::public("getVotingPowerEmaAlpha(uint16)")]
    #[precompile::view]
    fn get_voting_power_ema_alpha(_: &mut impl PrecompileHandle, netuid: u16) -> EvmResult<u64> {
        Ok(pallet_subtensor::VotingPowerEmaAlpha::<R>::get(
            NetUid::from(netuid),
        ))
    }
}
