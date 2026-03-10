// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.0;

address constant IDRAND_ADDRESS = 0x0000000000000000000000000000000000000811;

/**
 * @title IDrand
 * @dev Precompile at 0x811 providing typed views into drand randomness beacon storage.
 */
interface IDrand {
    /// @dev Randomness beacon pulse data for a given round.
    struct DrandPulse {
        bytes32 randomness;
        bytes signature;
    }

    /// @dev Drand beacon configuration.
    ///      Mirrors the full on-chain `BeaconConfiguration` storage struct.
    ///      `isExplicitlyConfigured` is `false` when the beacon storage key has never been
    ///      explicitly written; the pallet then operates with its hardcoded Quicknet default
    ///      (i.e. the other fields still contain valid Quicknet values, not zeroes).
    struct BeaconConfig {
        uint32  genesisTime;
        uint32  period;
        bytes   publicKey;           // 96-byte G2 public key (no bytes96 in Solidity)
        bytes32 chainHash;           // 32-byte chain hash
        bytes32 groupHash;           // 32-byte group hash
        bytes   schemeId;            // scheme identifier, UTF-8 string bytes (e.g. "bls-unchained-g1-rfc9380")
        bytes   beaconId;            // beacon identifier, UTF-8 string bytes (e.g. "quicknet")
        bool    isExplicitlyConfigured;
    }

    /// @dev Returns the last stored drand round number.
    function getLastStoredRound() external view returns (uint64);

    /// @dev Returns the oldest stored drand round number.
    function getOldestStoredRound() external view returns (uint64);

    /// @dev Returns the pulse (randomness, signature) for a specific round.
    ///      Returns empty bytes for both fields if the round is not found.
    /// @param round The drand round number.
    function getPulse(uint64 round) external view returns (DrandPulse memory);

    /// @dev Returns the randomness from the latest stored round as bytes32.
    ///      Returns zero bytes32 if no pulse is stored.
    function getCurrentRandomness() external view returns (bytes32);

    /// @dev Returns the drand beacon configuration.
    ///      isConfigured is false when the beacon has never been explicitly set.
    function getBeaconConfig() external view returns (BeaconConfig memory);

    /// @dev Returns whether a specific migration has run.
    /// @param migrationName The migration key exactly as stored on-chain (e.g. "migrate_set_oldest_round").
    function getHasMigrationRun(string memory migrationName) external view returns (bool);

    /// @dev Returns the block number at which the next unsigned transaction will be accepted.
    function getNextUnsignedAt() external view returns (uint64);

    /// @dev Returns the current pallet version (major version number).
    function getPalletVersion() external view returns (uint16);
}
