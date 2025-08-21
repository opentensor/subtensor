pragma solidity ^0.8.0;

address constant IPROXY_ADDRESS = 0x000000000000000000000000000000000000080b;

interface IProxy {
   /**
    * @dev Executes a proxied call on behalf of the `real` account.
    *
    * This function allows the proxy account (derived from sender) to execute a call as the `real` account
    * via the proxy pallet. The sender must be a registered proxy for the `real` account.
    *
    * @param real The real account public key (32 bytes).
    * @param force_proxy_type The proxy type to force (0 for none).
    * @param call The encoded RuntimeCall to dispatch.
    *
    * Requirements:
    * - Sender must be a valid proxy for the `real` account.
    * - Proxy type must permit the call.
    */
    function proxy(
        bytes32 real,
        uint256 force_proxy_type,
        bytes calldata call
    ) external payable;
}
