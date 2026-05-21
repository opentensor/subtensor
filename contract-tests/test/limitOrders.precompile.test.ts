import * as assert from "assert";

import { ethers } from "ethers";
import { TypedApi } from "polkadot-api";
import { devnet } from "@polkadot-api/descriptors";
import {
  buildOrderInput,
  ILIMITORDERS_ADDRESS,
  ILimitOrdersABI,
} from "../src/contracts/limitOrders";
import { generateRandomEthersWallet } from "../src/utils";
import { getDevnetApi } from "../src/substrate";
import { forceSetBalanceToEthAddress } from "../src/subtensor";

describe("Limit orders precompile E2E smoke", () => {
  let api: TypedApi<typeof devnet>;
  let wallet1: ethers.Wallet;
  let wallet2: ethers.Wallet;
  let limitOrdersContract: ethers.Contract;

  beforeEach(async () => {
    api = await getDevnetApi();

    wallet1 = generateRandomEthersWallet();
    wallet2 = generateRandomEthersWallet();
    limitOrdersContract = new ethers.Contract(
      ILIMITORDERS_ADDRESS,
      ILimitOrdersABI,
      wallet1,
    );

    await forceSetBalanceToEthAddress(api, wallet1.address);
    await forceSetBalanceToEthAddress(api, wallet2.address);
  });

  it("reads pallet status through the precompile", async () => {
    const enabled = await limitOrdersContract.getLimitOrdersEnabled();
    assert.strictEqual(enabled, true);
  });

  it("derives order ids and cancels orders through the precompile", async () => {
    const order = buildOrderInput(wallet1.address, wallet2.address);

    const orderId = await limitOrdersContract.deriveOrderId(order);
    assert.strictEqual(await limitOrdersContract.getOrderStatus(orderId), 0);

    const tx = await limitOrdersContract.cancelOrder(order);
    await tx.wait();

    assert.strictEqual(await limitOrdersContract.getOrderStatus(orderId), 3);
  });
});
