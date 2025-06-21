pragma solidity ^0.8.0;

address constant IPURE_PROXY_ADDRESS = 0x0000000000000000000000000000000000000809;

interface IPureProxy {
    function createPureProxy() external returns (bytes32);

    function pureProxyCall(uint8[] memory call) external;

    function getPureProxy() external view returns (bytes32);
}
