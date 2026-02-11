// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/// @title INeuronRegistrationCost
/// @notice Interface for querying neuron registration costs on Bittensor subnets
/// @dev Accessible at address 0x0000000000000000000000000000000000000806
interface INeuronRegistrationCost {
    /// @notice Get the current burn cost to register a neuron on a subnet
    /// @dev The cost can change dynamically based on network activity
    /// @param netuid The subnet identifier
    /// @return The burn cost in RAO (1 TAO = 10^9 RAO)
    function getBurn(uint16 netuid) external view returns (uint256);

    /// @notice Get the minimum burn cost for a subnet
    /// @param netuid The subnet identifier
    /// @return The minimum burn cost in RAO
    function getMinBurn(uint16 netuid) external view returns (uint256);

    /// @notice Get the maximum burn cost for a subnet
    /// @param netuid The subnet identifier
    /// @return The maximum burn cost in RAO
    function getMaxBurn(uint16 netuid) external view returns (uint256);

    /// @notice Check if registration is allowed on a subnet
    /// @param netuid The subnet identifier
    /// @return True if registration is allowed
    function isRegistrationAllowed(uint16 netuid) external view returns (bool);
}
