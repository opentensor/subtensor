pragma solidity ^0.8.0;

/// @custom:deprecated This precompile (V1) uses ETH-precision values.
/// Use IStaking at 0x0000000000000000000000000000000000000805 (V2) instead,
/// which uses RAO-precision values aligned with the Substrate API.
address constant ISTAKING_V1_ADDRESS = 0x0000000000000000000000000000000000000801;

interface IStakingV1 {
    /**
     * @dev Adds a subtensor stake corresponding to the value sent with the transaction, associated
     * with the `hotkey`.
     *
     * @param hotkey The hotkey public key (32 bytes).
     * @param netuid The subnet to stake to (uint256).
     *
     * @deprecated Use IStaking.addStake at 0x805 instead. V1 uses ETH-precision values.
     */
    function addStake(bytes32 hotkey, uint256 netuid) external payable;

    /**
     * @dev Removes a subtensor stake `amount` from the specified `hotkey`.
     *
     * @param hotkey The hotkey public key (32 bytes).
     * @param amount The amount to unstake in rao.
     * @param netuid The subnet to stake to (uint256).
     *
     * @deprecated Use IStaking.removeStake at 0x805 instead. V1 uses ETH-precision values.
     */
    function removeStake(
        bytes32 hotkey,
        uint256 amount,
        uint256 netuid
    ) external;

    /**
     * @dev Returns the amount of RAO staked by the coldkey.
     *
     * @param coldkey The coldkey public key (32 bytes).
     * @return The amount of RAO staked by the coldkey.
     *
     * @deprecated Use IStaking.getTotalColdkeyStake at 0x805 instead.
     */
    function getTotalColdkeyStake(
        bytes32 coldkey
    ) external view returns (uint256);

    /**
     * @dev Returns the total amount of stake under a hotkey (delegative or otherwise).
     *
     * @param hotkey The hotkey public key (32 bytes).
     * @return The total amount of RAO staked under the hotkey.
     *
     * @deprecated Use IStaking.getTotalHotkeyStake at 0x805 instead.
     */
    function getTotalHotkeyStake(
        bytes32 hotkey
    ) external view returns (uint256);

    /**
     * @dev Delegates staking to a proxy account.
     *
     * @param delegate The public key (32 bytes) of the delegate.
     *
     * @deprecated Use IStaking.addProxy at 0x805 instead.
     */
    function addProxy(bytes32 delegate) external;

    /**
     * @dev Removes staking proxy account.
     *
     * @param delegate The public key (32 bytes) of the delegate.
     *
     * @deprecated Use IStaking.removeProxy at 0x805 instead.
     */
    function removeProxy(bytes32 delegate) external;

    /**
     * @dev Returns the stake amount associated with the specified `hotkey` and `coldkey`.
     *
     * @param hotkey The hotkey public key (32 bytes).
     * @param coldkey The coldkey public key (32 bytes).
     * @param netuid The subnet the stake is on (uint256).
     * @return The current stake amount in uint256 format.
     *
     * @deprecated Use IStaking.getStake at 0x805 instead. V1 returns ETH-precision values.
     */
    function getStake(
        bytes32 hotkey,
        bytes32 coldkey,
        uint256 netuid
    ) external view returns (uint256);
}
