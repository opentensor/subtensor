pragma solidity ^0.8.0;

address constant IUID_LOOKUP_ADDRESS = 0x0000000000000000000000000000000000000807;

interface IRuntimeCallQuery {
    function call(
        bytes memory call,
    ) external payable;

    function query(
        bytes memory key,
    ) external view returns (bytes memory result);
}

