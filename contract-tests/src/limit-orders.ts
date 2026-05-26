import { devnet } from "@polkadot-api/descriptors";
import { KeyPair } from "@polkadot-labs/hdkd-helpers";
import { blake2AsHex } from "@polkadot/util-crypto";
import { Binary, getTypedCodecs, TypedApi } from "polkadot-api";
import type { SS58String } from "polkadot-api/ss58";

import { convertPublicKeyToSs58 } from "./address-utils";
import {
  buildOrderInput,
  FAR_FUTURE,
  OrderInput,
  SignedOrderInput,
} from "./contracts/limitOrders";
import {
  getAliceSigner,
  getCharlieSigner,
  getSignerFromKeypair,
  waitForTransactionWithRetry,
} from "./substrate";
import { forceSetBalanceToSs58Address, setSubtokenEnable } from "./subtensor";

export type OrderType = "LimitBuy" | "TakeProfit" | "StopLoss";

export interface SubstrateOrder {
  signer: string;
  hotkey: string;
  netuid: number;
  order_type: OrderType;
  amount: bigint;
  limit_price: bigint;
  expiry: bigint;
  fee_rate: number;
  fee_recipient: string;
  relayer: string[] | null;
  max_slippage: number | null;
  chain_id: bigint;
  partial_fills_enabled: boolean;
}

export interface SubstrateVersionedOrder {
  V1: SubstrateOrder;
}

export interface SubstrateSignedOrder {
  order: SubstrateVersionedOrder;
  signature: { Sr25519: `0x${string}` };
  partial_fill: number | null;
}

type VersionedOrderCodec = Awaited<
  ReturnType<typeof getTypedCodecs<typeof devnet>>
>["tx"]["LimitOrders"]["execute_orders"]["inner"]["orders"]["inner"]["inner"]["order"];

let versionedOrderCodec: VersionedOrderCodec | undefined;

async function getVersionedOrderCodec(): Promise<VersionedOrderCodec> {
  if (versionedOrderCodec === undefined) {
    const codec = await getTypedCodecs(devnet);
    versionedOrderCodec =
      codec.tx.LimitOrders.execute_orders.inner.orders.inner.inner.order;
  }
  return versionedOrderCodec;
}

function toPapiVersionedOrder(order: SubstrateVersionedOrder) {
  const inner = order.V1;
  return {
    type: "V1" as const,
    value: {
      signer: inner.signer as SS58String,
      hotkey: inner.hotkey as SS58String,
      netuid: inner.netuid,
      order_type: { type: inner.order_type, value: undefined },
      amount: inner.amount,
      limit_price: inner.limit_price,
      expiry: inner.expiry,
      fee_rate: inner.fee_rate,
      fee_recipient: inner.fee_recipient as SS58String,
      relayer: inner.relayer?.map((account) => account as SS58String),
      max_slippage: inner.max_slippage ?? undefined,
      chain_id: inner.chain_id,
      partial_fills_enabled: inner.partial_fills_enabled,
    },
  };
}

function toPapiSignedOrder(order: SubstrateSignedOrder) {
  return {
    order: toPapiVersionedOrder(order.order),
    signature: {
      type: "Sr25519" as const,
      value: Binary.fromHex(order.signature.Sr25519),
    },
    partial_fill: order.partial_fill ?? undefined,
  };
}

export async function fetchChainId(
  api: TypedApi<typeof devnet>,
): Promise<bigint> {
  return await api.query.EVMChainId.ChainId.getValue();
}

export async function ensureLimitOrdersEnabled(
  api: TypedApi<typeof devnet>,
): Promise<void> {
  const enabled = await api.query.LimitOrders.LimitOrdersEnabled.getValue();
  if (enabled) {
    return;
  }

  const alice = getAliceSigner();
  const tx = api.tx.Sudo.sudo({
    call: api.tx.LimitOrders.set_pallet_status({ enabled: true }).decodedCall,
  });
  await waitForTransactionWithRetry(api, tx, alice);
}

export async function setupLimitOrderSubnet(
  api: TypedApi<typeof devnet>,
  netuid: number,
): Promise<void> {
  await setSubtokenEnable(api, netuid, true);
}

export async function buildSubstrateSignedOrder(
  api: TypedApi<typeof devnet>,
  params: {
    signer: KeyPair;
    hotkey: string;
    netuid: number;
    orderType: OrderType;
    amount: bigint;
    limitPrice: bigint;
    expiry: bigint;
    feeRate: number;
    feeRecipient: string;
    chainId: bigint;
    relayer?: string[] | null;
    maxSlippage?: number | null;
    partialFillsEnabled?: boolean;
  },
): Promise<SubstrateSignedOrder> {
  void api;
  const inner: SubstrateOrder = {
    signer: convertPublicKeyToSs58(params.signer.publicKey),
    hotkey: params.hotkey,
    netuid: params.netuid,
    order_type: params.orderType,
    amount: params.amount,
    limit_price: params.limitPrice,
    expiry: params.expiry,
    fee_rate: params.feeRate,
    fee_recipient: params.feeRecipient,
    relayer: params.relayer ?? null,
    max_slippage: params.maxSlippage ?? null,
    chain_id: params.chainId,
    partial_fills_enabled: params.partialFillsEnabled ?? false,
  };

  const versionedOrder: SubstrateVersionedOrder = { V1: inner };
  const orderCodec = await getVersionedOrderCodec();
  const encoded = orderCodec.enc(toPapiVersionedOrder(versionedOrder));
  const sig = params.signer.sign(encoded);

  return {
    order: versionedOrder,
    signature: {
      Sr25519: (`0x${Buffer.from(sig).toString("hex")}`) as `0x${string}`,
    },
    partial_fill: null,
  };
}

export async function orderIdFromVersionedOrder(
  api: TypedApi<typeof devnet>,
  order: SubstrateVersionedOrder,
): Promise<`0x${string}`> {
  void api;
  const orderCodec = await getVersionedOrderCodec();
  const encoded = orderCodec.enc(toPapiVersionedOrder(order));
  return blake2AsHex(encoded, 256) as `0x${string}`;
}

export function toPrecompileSignedOrderInput(
  order: OrderInput,
  signatureHex: string,
  partialFill?: bigint,
): SignedOrderInput {
  const normalized = signatureHex.startsWith("0x")
    ? signatureHex
    : `0x${signatureHex}`;

  return {
    order,
    signature: normalized,
    has_partial_fill: partialFill !== undefined,
    partial_fill: partialFill ?? BigInt(0),
  };
}

export function buildInvalidSignedOrderInput(
  signerAddress: string,
  hotkeyAddress: string,
  chainId: bigint,
): SignedOrderInput {
  const order = buildOrderInput(signerAddress, hotkeyAddress, { chain_id: chainId });
  // sr25519 signatures are 64 bytes (128 hex chars).
  return toPrecompileSignedOrderInput(order, `0x${"00".repeat(64)}`);
}

export async function associateHotkey(
  api: TypedApi<typeof devnet>,
  coldkey: KeyPair,
  hotkeySs58: string,
): Promise<void> {
  const signer = getSignerFromKeypair(coldkey);
  const tx = api.tx.SubtensorModule.try_associate_hotkey({ hotkey: hotkeySs58 });
  await waitForTransactionWithRetry(api, tx, signer);
}

export async function prepareBuyerForLimitBuy(
  api: TypedApi<typeof devnet>,
  buyer: KeyPair,
  netuid: number,
  hotkeySs58: string,
): Promise<void> {
  await setupLimitOrderSubnet(api, netuid);
  await forceSetBalanceToSs58Address(
    api,
    convertPublicKeyToSs58(buyer.publicKey),
  );
  await associateHotkey(api, buyer, hotkeySs58);
}

export async function executeSignedOrdersViaSubstrate(
  api: TypedApi<typeof devnet>,
  orders: SubstrateSignedOrder[],
): Promise<void> {
  const charlie = getCharlieSigner();
  const tx = api.tx.LimitOrders.execute_orders({
    orders: orders.map(toPapiSignedOrder),
  });
  await waitForTransactionWithRetry(api, tx, charlie);
}

export { FAR_FUTURE };
