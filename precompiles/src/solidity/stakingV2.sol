pragma solidity ^0.8.0;

address constant ISTAKING_ADDRESS = 0x0000000000000000000000000000000000000805;

interface IStaking {
    /**
     * @dev Adds a subtensor stake `amount` associated with the `hotkey`.
     *
     * This function allows external accounts and contracts to stake TAO into the subtensor pallet,
     * which effectively calls `add_stake` on the subtensor pallet with specified hotkey as a parameter
     * and coldkey being the hashed address mapping of H160 sender address to Substrate ss58 address as
     * implemented in Frontier HashedAddressMapping:
     * https://github.com/polkadot-evm/frontier/blob/2e219e17a526125da003e64ef22ec037917083fa/frame/evm/src/lib.rs#L739
     *
     * @param hotkey The hotkey public key (32 bytes).
     * @param amount The amount to stake in rao.
     * @param netuid The subnet to stake to (uint256).
     *
     * Requirements:
     * - `hotkey` must be a valid hotkey registered on the network, ensuring that the stake is
     *   correctly attributed.
     */
    function addStake(
        bytes32 hotkey,
        uint256 amount,
        uint256 netuid
    ) external payable;

    /**
     * @dev Removes a subtensor stake `amount` from the specified `hotkey`.
     *
     * This function allows external accounts and contracts to unstake TAO from the subtensor pallet,
     * which effectively calls `remove_stake` on the subtensor pallet with specified hotkey as a parameter
     * and coldkey being the hashed address mapping of H160 sender address to Substrate ss58 address as
     * implemented in Frontier HashedAddressMapping:
     * https://github.com/polkadot-evm/frontier/blob/2e219e17a526125da003e64ef22ec037917083fa/frame/evm/src/lib.rs#L739
     *
     * @param hotkey The hotkey public key (32 bytes).
     * @param amount The amount to unstake in alpha.
     * @param netuid The subnet to stake to (uint256).
     *
     * Requirements:
     * - `hotkey` must be a valid hotkey registered on the network, ensuring that the stake is
     *   correctly attributed.
     * - The existing stake amount must be not lower than specified amount
     */
    function removeStake(
        bytes32 hotkey,
        uint256 amount,
        uint256 netuid
    ) external;

    /**
     * @dev Moves a subtensor stake `amount` associated with the `hotkey` to a different hotkey
     * `destination_hotkey`.
     *
     * This function allows external accounts and contracts to move staked TAO from one hotkey to another,
     * which effectively calls `move_stake` on the subtensor pallet with specified origin and destination
     * hotkeys as parameters being the hashed address mappings of H160 sender address to Substrate ss58
     * address as implemented in Frontier HashedAddressMapping:
     * https://github.com/polkadot-evm/frontier/blob/2e219e17a526125da003e64ef22ec037917083fa/frame/evm/src/lib.rs#L739
     *
     * @param origin_hotkey The origin hotkey public key (32 bytes).
     * @param destination_hotkey The destination hotkey public key (32 bytes).
     * @param origin_netuid The subnet to move stake from (uint256).
     * @param destination_netuid The subnet to move stake to (uint256).
     * @param amount The amount to move in rao.
     *
     * Requirements:
     * - `origin_hotkey` and `destination_hotkey` must be valid hotkeys registered on the network, ensuring
     * that the stake is correctly attributed.
     */
    function moveStake(
        bytes32 origin_hotkey,
        bytes32 destination_hotkey,
        uint256 origin_netuid,
        uint256 destination_netuid,
        uint256 amount
    ) external;

    /**
     * @dev Transfer a subtensor stake `amount` associated with the transaction signer to a different coldkey
     * `destination_coldkey`.
     *
     * This function allows external accounts and contracts to transfer staked TAO to another coldkey,
     * which effectively calls `transfer_stake` on the subtensor pallet with specified destination
     * coldkey as a parameter being the hashed address mapping of H160 sender address to Substrate ss58
     * address as implemented in Frontier HashedAddressMapping:
     * https://github.com/polkadot-evm/frontier/blob/2e219e17a526125da003e64ef22ec037917083fa/frame/evm/src/lib.rs#L739
     *
     * @param destination_coldkey The destination coldkey public key (32 bytes).
     * @param hotkey The hotkey public key (32 bytes).
     * @param origin_netuid The subnet to move stake from (uint256).
     * @param destination_netuid The subnet to move stake to (uint256).
     * @param amount The amount to move in rao.
     *
     * Requirements:
     * - `origin_hotkey` and `destination_hotkey` must be valid hotkeys registered on the network, ensuring
     * that the stake is correctly attributed.
     */
    function transferStake(
        bytes32 destination_coldkey,
        bytes32 hotkey,
        uint256 origin_netuid,
        uint256 destination_netuid,
        uint256 amount
    ) external;

    /**
     * @dev Returns the amount of RAO staked by the coldkey.
     *
     * This function allows external accounts and contracts to query the amount of RAO staked by the coldkey
     * which effectively calls `get_total_coldkey_stake` on the subtensor pallet with
     * specified coldkey as a parameter.
     *
     * @param coldkey The coldkey public key (32 bytes).
     * @return The amount of RAO staked by the coldkey.
     */
    function getTotalColdkeyStake(
        bytes32 coldkey
    ) external view returns (uint256);

    /**
     * @dev Returns the total amount of stake under a hotkey (delegative or otherwise)
     *
     * This function allows external accounts and contracts to query the total amount of RAO staked under a hotkey
     * which effectively calls `get_total_hotkey_stake` on the subtensor pallet with
     * specified hotkey as a parameter.
     *
     * @param hotkey The hotkey public key (32 bytes).
     * @return The total amount of RAO staked under the hotkey.
     */
    function getTotalHotkeyStake(
        bytes32 hotkey
    ) external view returns (uint256);

    /**
     * @dev Returns the stake amount associated with the specified `hotkey` and `coldkey`.
     *
     * This function retrieves the current stake amount linked to a specific hotkey and coldkey pair.
     * It is a view function, meaning it does not modify the state of the contract and is free to call.
     *
     * @param hotkey The hotkey public key (32 bytes).
     * @param coldkey The coldkey public key (32 bytes).
     * @param netuid The subnet the stake is on (uint256).
     * @return The current stake amount in uint256 format.
     */
    function getStake(
        bytes32 hotkey,
        bytes32 coldkey,
        uint256 netuid
    ) external view returns (uint256);

    /**
     * @dev Delegates staking to a proxy account.
     *
     * @param delegate The public key (32 bytes) of the delegate.
     */
    function addProxy(bytes32 delegate) external;

    /**
     * @dev Removes staking proxy account.
     *
     * @param delegate The public key (32 bytes) of the delegate.
     */
    function removeProxy(bytes32 delegate) external;

    /**
     * @dev Returns the validators that have staked alpha under a hotkey.
     *
     * This function retrieves the validators that have staked alpha under a specific hotkey.
     * It is a view function, meaning it does not modify the state of the contract and is free to call.
     *
     * @param hotkey The hotkey public key (32 bytes).
     * @param netuid The subnet the stake is on (uint256).
     * @return An array of validators that have staked alpha under the hotkey.
     */
    function getAlphaStakedValidators(
        bytes32 hotkey,
        uint256 netuid
    ) external view returns (uint256[] memory);

    /**
     * @dev Returns the total amount of alpha staked under a hotkey.
     *
     * This function retrieves the total amount of alpha staked under a specific hotkey.
     * It is a view function, meaning it does not modify the state of the contract and is free to call.
     *
     * @param hotkey The hotkey public key (32 bytes).
     * @param netuid The subnet the stake is on (uint256).
     * @return The total amount of alpha staked under the hotkey.
     */
    function getTotalAlphaStaked(
        bytes32 hotkey,
        uint256 netuid
    ) external view returns (uint256);

    /**
     * @dev Adds a subtensor stake `amount` associated with the `hotkey` within a price limit.
     *
     * This function allows external accounts and contracts to stake TAO into the subtensor pallet,
     * which effectively calls `add_stake_limit` on the subtensor pallet with specified hotkey as a parameter
     * and coldkey being the hashed address mapping of H160 sender address to Substrate ss58 address as
     * implemented in Frontier HashedAddressMapping:
     * https://github.com/polkadot-evm/frontier/blob/2e219e17a526125da003e64ef22ec037917083fa/frame/evm/src/lib.rs#L739
     *
     * @param hotkey The hotkey public key (32 bytes).
     * @param amount The amount to stake in rao.
     * @param limit_price The price limit to stake at in rao. Number of rao per alpha.
     * @param allow_partial Whether to allow partial stake.
     * @param netuid The subnet to stake to (uint256).
     *
     * Requirements:
     * - `hotkey` must be a valid hotkey registered on the network, ensuring that the stake is
     *   correctly attributed.
     */
    function addStakeLimit(
        bytes32 hotkey,
        uint256 amount,
        uint256 limit_price,
        bool allow_partial,
        uint256 netuid
    ) external payable;

    /**
     * @dev Removes a subtensor stake `amount` from the specified `hotkey` within a price limit.
     *
     * This function allows external accounts and contracts to unstake TAO from the subtensor pallet,
     * which effectively calls `remove_stake_limit` on the subtensor pallet with specified hotkey as a parameter
     * and coldkey being the hashed address mapping of H160 sender address to Substrate ss58 address as
     * implemented in Frontier HashedAddressMapping:
     * https://github.com/polkadot-evm/frontier/blob/2e219e17a526125da003e64ef22ec037917083fa/frame/evm/src/lib.rs#L739
     *
     * @param hotkey The hotkey public key (32 bytes).
     * @param amount The amount to unstake in alpha.
     * @param limit_price The price limit to unstake at in rao. Number of rao per alpha.
     * @param allow_partial Whether to allow partial unstake.
     * @param netuid The subnet to stake to (uint256).
     *
     * Requirements:
     * - `hotkey` must be a valid hotkey registered on the network, ensuring that the stake is
     *   correctly attributed.
     * - The existing stake amount must be not lower than specified amount
     */
    function removeStakeLimit(
        bytes32 hotkey,
        uint256 amount,
        uint256 limit_price,
        bool allow_partial,
        uint256 netuid
    ) external;
}
