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
    function addStake(bytes32 hotkey, uint256 amount, uint256 netuid) external payable;

    /**
     * @dev Swaps a subtensor stake from one subnet to another.
     *
     * This function allows external accounts and contracts to swap a subtensor stake between two different subnets.
     * It effectively calls `swap_stake` on the subtensor pallet with specified hotkey, origin subnet, destination subnet,
     * and alpha amount as parameters.
     *
     * @param hotkey The hotkey public key (32 bytes).
     * @param originNetuid The subnet to swap from (uint256).
     * @param destinationNetuid The subnet to swap to (uint256).
     * @param alphaAmount The amount to swap in rao
     *
     * Requirements:
     * - `hotkey` must be a valid hotkey registered on the network, ensuring that the stake is
     *   correctly attributed.
     * - `originNetuid` and `destinationNetuid` must be valid subnets on the network.
     */
    function swapStake(
      bytes32 hotkey,
      uint256 originNetuid,
      uint256 destinationNetuid,
      uint256 alphaAmount
    ) external;

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
     * @dev Transfers a subtensor stake from one hotkey to another.
     *
     * This function allows external accounts and contracts to transfer a subtensor stake from one hotkey to another.
     * It effectively calls `transfer_stake` on the subtensor pallet with specified destination coldkey, hotkey, origin subnet,
     * destination subnet, and alpha amount as parameters.
     *
     * @param destinationColdkey The coldkey public key (32 bytes) of the destination account.
     * @param hotkey The hotkey public key (32 bytes) of the source account.
     * @param originNetuid The subnet to transfer from (uint256).
     * @param destinationNetuid The subnet to transfer to (uint256).
     * @param alphaAmount The amount to transfer in rao.
     *
     * Requirements:
     * - `hotkey` must be a valid hotkey registered on the network, ensuring that the stake is
     *   correctly attributed.
     * - `destinationColdkey` must be a valid coldkey registered on the network, ensuring that the stake is
     *   correctly attributed.
     * - `originNetuid` and `destinationNetuid` must be valid subnets on the network.
     */
    function transferStake(
        bytes32 destinationColdkey,
        bytes32 hotkey,
        uint256 originNetuid,
        uint256 destinationNetuid,
        uint256 alphaAmount
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
    function getTotalColdkeyStake(bytes32 coldkey) external view returns (uint256);

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
    function getTotalHotkeyStake(bytes32 hotkey) external view returns (uint256);

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
}
