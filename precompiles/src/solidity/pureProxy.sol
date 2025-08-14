// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

address constant IPROXY_ADDRESS = 0x000000000000000000000000000000000000080b;

interface IProxy {
    function createPureProxy() external;

    function proxyCall(bytes32 real, uint8[] memory call) external;

    function getPureProxy() external view returns (bytes32[] memory);

    function killPureProxy(bytes32 proxy) external;
}
