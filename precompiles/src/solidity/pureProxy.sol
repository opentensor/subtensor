pragma solidity ^0.8.0;

address constant IPURE_PROXY_ADDRESS = 0x000000000000000000000000000000000000080a;

interface IPureProxy {
    function createPureProxy() external returns (bytes32);

    function pureProxyCall(uint8[] memory call) external;

    function getPureProxy() external view returns (bytes32);
}
