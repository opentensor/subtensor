pragma solidity ^0.8.0;

address constant IDRAND_ADDRESS = 0x0000000000000000000000000000000000000811;

/**
 * @title IDrand
 * @dev Precompile at 0x811 providing typed views into drand randomness beacon storage.
 */
interface IDrand {
    /// @dev Returns the last stored drand round number.
    function getLastStoredRound() external view returns (uint64);

    /// @dev Returns the oldest stored drand round number.
    function getOldestStoredRound() external view returns (uint64);

    /// @dev Returns the pulse (randomness, signature) for a specific round.
    /// @param round The drand round number.
    /// @return randomness The 32-byte randomness value.
    /// @return signature The BLS signature for this round.
    function getPulse(uint64 round) external view returns (bytes memory randomness, bytes memory signature);

    /// @dev Returns the randomness from the latest stored round as bytes32.
    function getCurrentRandomness() external view returns (bytes32);
}
