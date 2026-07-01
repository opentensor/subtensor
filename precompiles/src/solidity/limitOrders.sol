pragma solidity ^0.8.0;

address constant ILIMITORDERS_ADDRESS = 0x000000000000000000000000000000000000080e;

struct OrderInput {
    address signer;
    address hotkey;
    uint16 netuid;
    uint8 order_type;
    uint64 amount;
    uint64 limit_price;
    uint64 expiry;
    uint32 fee_rate;
    address fee_recipient;
    address[] relayer;
    bool has_max_slippage;
    uint32 max_slippage;
    uint64 chain_id;
    bool partial_fills_enabled;
}

struct SignedOrderInput {
    OrderInput order;
    bytes signature;
    bool has_partial_fill;
    uint64 partial_fill;
}

interface ILimitOrders {
    /**
     * @dev Returns whether the limit orders pallet is enabled.
     */
    function getLimitOrdersEnabled() external view returns (bool);

    /**
     * @dev Returns the on-chain status for an order id.
     * 0 = none, 1 = fulfilled, 2 = partially filled, 3 = cancelled.
     */
    function getOrderStatus(bytes32 orderId) external view returns (uint8);

    /**
     * @dev Derives the order id from an order payload.
     */
    function deriveOrderId(OrderInput calldata order) external view returns (bytes32);

    /**
     * @dev Executes a batch of signed limit orders.
     * The EVM caller is treated as the relayer.
     * @param shouldFail When true, the first order failure reverts the whole batch.
     */
    function executeOrders(
        SignedOrderInput[] calldata orders,
        bool shouldFail
    ) external payable;

    /**
     * @dev Executes signed limit orders for a single subnet.
     * The EVM caller is treated as the relayer.
     */
    function executeBatchedOrders(
        uint16 netuid,
        SignedOrderInput[] calldata orders
    ) external payable;

    /**
     * @dev Registers a cancellation intent for an order.
     * The EVM caller must match the order signer.
     */
    function cancelOrder(OrderInput calldata order) external payable;
}
