// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

address constant IPURE_PROXY_ADDRESS = 0x000000000000000000000000000000000000080b;

interface IPureProxy {
    function createPureProxy() external returns (bytes32);

    function pureProxyCall(bytes32 proxy, uint8[] memory call) external;
}
