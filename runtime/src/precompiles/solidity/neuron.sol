pragma solidity ^0.8.0;

address constant ISTAKING_ADDRESS = 0x0000000000000000000000000000000000000805;

interface INeuron {
    function setWeights(
        uint16 netuid,
        bytes memory dests,
        bytes memory weights,
        uint64 versionKey
    ) external payable;
}
