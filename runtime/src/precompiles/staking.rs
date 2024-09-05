/// Staking precompile's goal is to allow interaction between EVM users and smart contracts and 
/// subtensor staking functionality, namely add_stake, and remove_stake extrinsics.
/// 
/// Additional requirement is to preserve compatibility with Ethereum indexers, which requires 
/// no balance transfers from EVM accounts without a corresponding transaction that can be 
/// parsed by an indexer.
/// 
/// Implementation of add_stake:
///   - User transfers balance that will be staked to the precompile address with a payable 
///     method addStake. This method also takes hotkey public key (bytes32) of the hotkey
///     that the stake should be assigned to.
///   - Precompile transfers the balance back to the signing address, and then invokes 
///     do_add_stake from subtensor pallet with signing origin that mmatches to HashedAddressMapping
///     of the message sender, which will effectively withdraw and stake balance from the message 
///     sender.
///   - Precompile checks the result of do_add_stake and, in case of a failure, reverts the transaction, 
///     and leaves the balance on the message sender account.
/// 
/// Implementation of remove_stake:
///   - User involkes removeStake method and specifies hotkey public key (bytes32) of the hotkey
///     to remove stake from, and the amount to unstake.
///   - Precompile calls do_remove_stake method of the subtensor pallet with the signing origin of message 
///     sender, which effectively unstakes the specified amount and credits it to the message sender
///   - Precompile checks the result of do_remove_stake and, in case of a failure, reverts the transaction.
/// 
