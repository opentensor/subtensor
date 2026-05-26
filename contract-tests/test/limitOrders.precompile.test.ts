import * as assert from "assert";

import { devnet } from "@polkadot-api/descriptors";
import { ethers } from "ethers";
import { TypedApi } from "polkadot-api";

import { convertH160ToSS58, convertPublicKeyToSs58 } from "../src/address-utils";
import {
  buildOrderInput,
  ILIMITORDERS_ADDRESS,
  ILimitOrdersABI,
} from "../src/contracts/limitOrders";
import {
  buildInvalidSignedOrderInput,
  buildSubstrateSignedOrder,
  ensureLimitOrdersEnabled,
  executeSignedOrdersViaSubstrate,
  fetchChainId,
  orderIdFromVersionedOrder,
  prepareBuyerForLimitBuy,
} from "../src/limit-orders";
import {
  getAlice,
  getCharlie,
  getDevnetApi,
} from "../src/substrate";
import { forceSetBalanceToEthAddress } from "../src/subtensor";
import { generateRandomEthersWallet } from "../src/utils";

const NETUID = 1;
const BUY_AMOUNT = BigInt(1_000_000_000);

async function readOrderStatus(
  contract: ethers.Contract,
  orderId: string,
): Promise<number> {
  return Number(await contract.getOrderStatus(orderId));
}

describe("Limit orders precompile E2E smoke", () => {
  let api: TypedApi<typeof devnet>;
  let chainId: bigint;
  let wallet1: ethers.Wallet;
  let wallet2: ethers.Wallet;
  let wallet3: ethers.Wallet;
  let limitOrdersContract: ethers.Contract;

  before(async () => {
    api = await getDevnetApi();
    await ensureLimitOrdersEnabled(api);
    chainId = await fetchChainId(api);
  });

  beforeEach(async () => {
    wallet1 = generateRandomEthersWallet();
    wallet2 = generateRandomEthersWallet();
    wallet3 = generateRandomEthersWallet();
    limitOrdersContract = new ethers.Contract(
      ILIMITORDERS_ADDRESS,
      ILimitOrdersABI,
      wallet1,
    );

    await forceSetBalanceToEthAddress(api, wallet1.address);
    await forceSetBalanceToEthAddress(api, wallet2.address);
    await forceSetBalanceToEthAddress(api, wallet3.address);
  });

  it("reads pallet status through getLimitOrdersEnabled", async () => {
    const enabled = await limitOrdersContract.getLimitOrdersEnabled();
    assert.strictEqual(enabled, true);
  });

  it("returns zero status for unknown orders via getOrderStatus", async () => {
    const unknownId = ethers.id("unknown-limit-order");
    assert.strictEqual(
      await readOrderStatus(limitOrdersContract, unknownId),
      0,
    );
  });

  it("derives stable order ids via deriveOrderId", async () => {
    const order = buildOrderInput(wallet1.address, wallet2.address, {
      chain_id: chainId,
    });

    const first = await limitOrdersContract.deriveOrderId(order);
    const second = await limitOrdersContract.deriveOrderId(order);
    assert.strictEqual(first, second);
    assert.strictEqual(await readOrderStatus(limitOrdersContract, first), 0);
  });

  it("matches deriveOrderId with substrate encoding for mapped EVM accounts", async () => {
    const orderInput = buildOrderInput(wallet1.address, wallet2.address, {
      chain_id: chainId,
      fee_recipient: wallet3.address,
      relayer: [wallet3.address],
      has_max_slippage: true,
      max_slippage: 10_000_000,
    });

    const substrateOrder = {
      V1: {
        signer: convertH160ToSS58(wallet1.address),
        hotkey: convertH160ToSS58(wallet2.address),
        netuid: NETUID,
        order_type: "LimitBuy" as const,
        amount: orderInput.amount,
        limit_price: orderInput.limit_price,
        expiry: orderInput.expiry,
        fee_rate: orderInput.fee_rate,
        fee_recipient: convertH160ToSS58(wallet3.address),
        relayer: [convertH160ToSS58(wallet3.address)],
        max_slippage: orderInput.max_slippage,
        chain_id: chainId,
        partial_fills_enabled: false,
      },
    };

    const precompileId = await limitOrdersContract.deriveOrderId(orderInput);
    const substrateId = await orderIdFromVersionedOrder(api, substrateOrder);
    assert.strictEqual(precompileId, substrateId);
  });

  it("registers cancellations through cancelOrder", async () => {
    const order = buildOrderInput(wallet1.address, wallet2.address, {
      chain_id: chainId,
    });
    const orderId = await limitOrdersContract.deriveOrderId(order);

    const tx = await limitOrdersContract.cancelOrder(order);
    await tx.wait();

    assert.strictEqual(await readOrderStatus(limitOrdersContract, orderId), 3);
  });

  it("rejects cancelOrder from a non-signer", async () => {
    const order = buildOrderInput(wallet1.address, wallet2.address, {
      chain_id: chainId,
    });
    const otherWalletContract = new ethers.Contract(
      ILIMITORDERS_ADDRESS,
      ILimitOrdersABI,
      wallet2,
    );

    await assert.rejects(
      otherWalletContract.cancelOrder(order),
      /revert|execution reverted/i,
    );
  });

  it("accepts empty batches through executeOrders", async () => {
    const tx = await limitOrdersContract.executeOrders([]);
    await tx.wait();
  });

  it("accepts empty batches through executeBatchedOrders", async () => {
    const tx = await limitOrdersContract.executeBatchedOrders(NETUID, []);
    await tx.wait();
  });

  it("dispatches executeOrders without fulfilling invalid signatures", async () => {
    const invalidOrder = buildInvalidSignedOrderInput(
      wallet1.address,
      wallet2.address,
      chainId,
    );
    const orderId = await limitOrdersContract.deriveOrderId(invalidOrder.order);

    const tx = await limitOrdersContract.executeOrders([invalidOrder]);
    await tx.wait();

    assert.strictEqual(await readOrderStatus(limitOrdersContract, orderId), 0);
  });

  it("reverts executeBatchedOrders on invalid signatures", async () => {
    const invalidOrder = buildInvalidSignedOrderInput(
      wallet1.address,
      wallet2.address,
      chainId,
    );

    await assert.rejects(
      limitOrdersContract.executeBatchedOrders(NETUID, [invalidOrder], {
        gasLimit: 10_000_000,
      }),
      /revert|execution reverted/i,
    );
  });

  it("reports fulfilled orders via getOrderStatus after substrate execution", async () => {
    const alice = getAlice();
    const charlie = getCharlie();
    const aliceSs58 = convertPublicKeyToSs58(alice.publicKey);

    await prepareBuyerForLimitBuy(api, alice, NETUID, aliceSs58);

    const signed = await buildSubstrateSignedOrder(api, {
      signer: alice,
      hotkey: aliceSs58,
      netuid: NETUID,
      orderType: "LimitBuy",
      amount: BUY_AMOUNT,
      limitPrice: BigInt("18446744073709551615"),
      expiry: BigInt("18446744073709551615"),
      feeRate: 0,
      feeRecipient: convertPublicKeyToSs58(charlie.publicKey),
      chainId,
    });
    const orderId = await orderIdFromVersionedOrder(api, signed.order);

    await executeSignedOrdersViaSubstrate(api, [signed]);

    assert.strictEqual(await readOrderStatus(limitOrdersContract, orderId), 1);
  });
});
