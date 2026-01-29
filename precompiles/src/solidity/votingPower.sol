pragma solidity ^0.8.0;

address constant IVOTING_POWER_ADDRESS = 0x000000000000000000000000000000000000080d;

interface IVotingPower {
    /// Get voting power for a hotkey on a subnet.
    ///
    /// Returns the EMA of stake for the hotkey, which represents its voting power.
    /// Returns 0 if:
    /// - The hotkey has no voting power entry
    /// - Voting power tracking is not enabled for the subnet
    /// - The hotkey is not registered on the subnet
    ///
    /// @param netuid The subnet identifier
    /// @param hotkey The hotkey account ID (bytes32)
    /// @return The voting power value (in RAO, same precision as stake)
    function getVotingPower(
        uint16 netuid,
        bytes32 hotkey
    ) external view returns (uint256);

    /// Check if voting power tracking is enabled for a subnet.
    ///
    /// @param netuid The subnet identifier
    /// @return True if voting power tracking is enabled
    function isVotingPowerTrackingEnabled(
        uint16 netuid
    ) external view returns (bool);

    /// Get the block at which voting power tracking will be disabled.
    ///
    /// Returns 0 if not scheduled for disabling.
    /// When non-zero, tracking continues until this block, then stops.
    ///
    /// @param netuid The subnet identifier
    /// @return The block number at which tracking will be disabled (0 if not scheduled)
    function getVotingPowerDisableAtBlock(
        uint16 netuid
    ) external view returns (uint64);

    /// Get total voting power for a subnet.
    ///
    /// Returns the sum of all voting power for all validators on the subnet.
    /// Useful for calculating voting thresholds (e.g., 51% quorum).
    ///
    /// @param netuid The subnet identifier
    /// @return The total voting power across all validators
    function getTotalVotingPower(
        uint16 netuid
    ) external view returns (uint256);
}
