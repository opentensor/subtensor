export const ILIMITORDERS_ADDRESS =
  "0x000000000000000000000000000000000000080e";

export const ILimitOrdersABI = [
  {
    inputs: [
      {
        components: [
          { internalType: "address", name: "signer", type: "address" },
          { internalType: "address", name: "hotkey", type: "address" },
          { internalType: "uint16", name: "netuid", type: "uint16" },
          { internalType: "uint8", name: "order_type", type: "uint8" },
          { internalType: "uint64", name: "amount", type: "uint64" },
          { internalType: "uint64", name: "limit_price", type: "uint64" },
          { internalType: "uint64", name: "expiry", type: "uint64" },
          { internalType: "uint32", name: "fee_rate", type: "uint32" },
          { internalType: "address", name: "fee_recipient", type: "address" },
          { internalType: "address[]", name: "relayer", type: "address[]" },
          { internalType: "bool", name: "has_max_slippage", type: "bool" },
          { internalType: "uint32", name: "max_slippage", type: "uint32" },
          { internalType: "uint64", name: "chain_id", type: "uint64" },
          {
            internalType: "bool",
            name: "partial_fills_enabled",
            type: "bool",
          },
        ],
        internalType: "struct OrderInput",
        name: "order",
        type: "tuple",
      },
    ],
    name: "cancelOrder",
    outputs: [],
    stateMutability: "payable",
    type: "function",
  },
  {
    inputs: [
      {
        components: [
          { internalType: "address", name: "signer", type: "address" },
          { internalType: "address", name: "hotkey", type: "address" },
          { internalType: "uint16", name: "netuid", type: "uint16" },
          { internalType: "uint8", name: "order_type", type: "uint8" },
          { internalType: "uint64", name: "amount", type: "uint64" },
          { internalType: "uint64", name: "limit_price", type: "uint64" },
          { internalType: "uint64", name: "expiry", type: "uint64" },
          { internalType: "uint32", name: "fee_rate", type: "uint32" },
          { internalType: "address", name: "fee_recipient", type: "address" },
          { internalType: "address[]", name: "relayer", type: "address[]" },
          { internalType: "bool", name: "has_max_slippage", type: "bool" },
          { internalType: "uint32", name: "max_slippage", type: "uint32" },
          { internalType: "uint64", name: "chain_id", type: "uint64" },
          {
            internalType: "bool",
            name: "partial_fills_enabled",
            type: "bool",
          },
        ],
        internalType: "struct OrderInput",
        name: "order",
        type: "tuple",
      },
    ],
    name: "deriveOrderId",
    outputs: [{ internalType: "bytes32", name: "", type: "bytes32" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      { internalType: "uint16", name: "netuid", type: "uint16" },
      {
        components: [
          {
            components: [
              { internalType: "address", name: "signer", type: "address" },
              { internalType: "address", name: "hotkey", type: "address" },
              { internalType: "uint16", name: "netuid", type: "uint16" },
              { internalType: "uint8", name: "order_type", type: "uint8" },
              { internalType: "uint64", name: "amount", type: "uint64" },
              {
                internalType: "uint64",
                name: "limit_price",
                type: "uint64",
              },
              { internalType: "uint64", name: "expiry", type: "uint64" },
              { internalType: "uint32", name: "fee_rate", type: "uint32" },
              {
                internalType: "address",
                name: "fee_recipient",
                type: "address",
              },
              {
                internalType: "address[]",
                name: "relayer",
                type: "address[]",
              },
              {
                internalType: "bool",
                name: "has_max_slippage",
                type: "bool",
              },
              {
                internalType: "uint32",
                name: "max_slippage",
                type: "uint32",
              },
              { internalType: "uint64", name: "chain_id", type: "uint64" },
              {
                internalType: "bool",
                name: "partial_fills_enabled",
                type: "bool",
              },
            ],
            internalType: "struct OrderInput",
            name: "order",
            type: "tuple",
          },
          { internalType: "bytes", name: "signature", type: "bytes" },
          {
            internalType: "bool",
            name: "has_partial_fill",
            type: "bool",
          },
          { internalType: "uint64", name: "partial_fill", type: "uint64" },
        ],
        internalType: "struct SignedOrderInput[]",
        name: "orders",
        type: "tuple[]",
      },
    ],
    name: "executeBatchedOrders",
    outputs: [],
    stateMutability: "payable",
    type: "function",
  },
  {
    inputs: [
      {
        components: [
          {
            components: [
              { internalType: "address", name: "signer", type: "address" },
              { internalType: "address", name: "hotkey", type: "address" },
              { internalType: "uint16", name: "netuid", type: "uint16" },
              { internalType: "uint8", name: "order_type", type: "uint8" },
              { internalType: "uint64", name: "amount", type: "uint64" },
              {
                internalType: "uint64",
                name: "limit_price",
                type: "uint64",
              },
              { internalType: "uint64", name: "expiry", type: "uint64" },
              { internalType: "uint32", name: "fee_rate", type: "uint32" },
              {
                internalType: "address",
                name: "fee_recipient",
                type: "address",
              },
              {
                internalType: "address[]",
                name: "relayer",
                type: "address[]",
              },
              {
                internalType: "bool",
                name: "has_max_slippage",
                type: "bool",
              },
              {
                internalType: "uint32",
                name: "max_slippage",
                type: "uint32",
              },
              { internalType: "uint64", name: "chain_id", type: "uint64" },
              {
                internalType: "bool",
                name: "partial_fills_enabled",
                type: "bool",
              },
            ],
            internalType: "struct OrderInput",
            name: "order",
            type: "tuple",
          },
          { internalType: "bytes", name: "signature", type: "bytes" },
          {
            internalType: "bool",
            name: "has_partial_fill",
            type: "bool",
          },
          { internalType: "uint64", name: "partial_fill", type: "uint64" },
        ],
        internalType: "struct SignedOrderInput[]",
        name: "orders",
        type: "tuple[]",
      },
      {
        internalType: "bool",
        name: "shouldFail",
        type: "bool",
      },
    ],
    name: "executeOrders",
    outputs: [],
    stateMutability: "payable",
    type: "function",
  },
  {
    inputs: [],
    name: "getLimitOrdersEnabled",
    outputs: [{ internalType: "bool", name: "", type: "bool" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [{ internalType: "bytes32", name: "orderId", type: "bytes32" }],
    name: "getOrderStatus",
    outputs: [{ internalType: "uint8", name: "", type: "uint8" }],
    stateMutability: "view",
    type: "function",
  },
] as const;

export type SignedOrderInput = {
  order: OrderInput;
  signature: string;
  has_partial_fill: boolean;
  partial_fill: bigint;
};

export type OrderInput = {
  signer: string;
  hotkey: string;
  netuid: number;
  order_type: number;
  amount: bigint;
  limit_price: bigint;
  expiry: bigint;
  fee_rate: number;
  fee_recipient: string;
  relayer: string[];
  has_max_slippage: boolean;
  max_slippage: number;
  chain_id: bigint;
  partial_fills_enabled: boolean;
};

export const FAR_FUTURE = BigInt("18446744073709551615");

export function buildOrderInput(
  signer: string,
  hotkey: string,
  overrides: Partial<OrderInput> = {},
): OrderInput {
  return {
    signer,
    hotkey,
    netuid: 1,
    order_type: 0,
    amount: BigInt(1_000),
    limit_price: BigInt(1_000_000_000),
    expiry: FAR_FUTURE,
    fee_rate: 0,
    fee_recipient: signer,
    relayer: [],
    has_max_slippage: false,
    max_slippage: 0,
    chain_id: BigInt(42),
    partial_fills_enabled: false,
    ...overrides,
  };
}
