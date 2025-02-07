pragma solidity ^0.8.0;

address constant INeuron_ADDRESS = 0x0000000000000000000000000000000000000804;

interface INeuron {
    /**
     * @dev Registers a neuron by calling `do_burned_registration` internally with the origin set to the ss58 mirror of the H160 address.
     * This allows the H160 to further call neuron-related methods and receive emissions.
     *
     * @param netuid The subnet to register the neuron to (uint16).
     * @param hotkey The hotkey public key (32 bytes).
     */
    function burnedRegister(uint16 netuid, bytes32 hotkey) external payable;

    /**
     * @dev Sets the weights for a neuron.
     *
     * @param netuid The subnet to set the weights for (uint16).
     * @param dests The destinations of the weights (uint16[]).
     * @param weights The weights to set (uint16[]).
     * @param versionKey The version key for the weights (uint64).
     */
    function setWeights(
        uint16 netuid,
        uint16[] memory dests,
        uint16[] memory weights,
        uint64 versionKey
    ) external payable;

    /**
     * @dev Commits the weights for a neuron.
     *
     * @param netuid The subnet to commit the weights for (uint16).
     * @param commitHash The commit hash for the weights (uint256).
     */
    function commitWeights(uint16 netuid, uint256 commitHash) external payable;

    /**
     * @dev Reveals the weights for a neuron.
     *
     * @param netuid The subnet to reveal the weights for (uint16).
     * @param uids The unique identifiers for the weights (uint16[]).
     * @param values The values of the weights (uint16[]).
     * @param salt The salt values for the weights (uint16[]).
     * @param versionKey The version key for the weights (uint64).
     */
    function revealWeights(
        uint16 netuid,
        uint16[] memory uids,
        uint16[] memory values,
        uint16[] memory salt,
        uint64 versionKey
    ) external payable;
}
