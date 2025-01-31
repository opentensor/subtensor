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

    function setWeights(
        uint16 netuid,
        uint16[] memory dests,
        uint16[] memory weights,
        uint64 versionKey
    ) external payable;

    function commitWeights(uint16 netuid, uint256 commitHash) external payable;

    function revealWeights(
        uint16 netuid,
        uint16[] memory uids,
        uint16[] memory values,
        uint16[] memory salt,
        uint64 versionKey
    ) external payable;
}
