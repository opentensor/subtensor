import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { readFile } from "node:fs/promises";
import type { PolkadotClient, TypedApi } from "polkadot-api";
import { hexToU8a } from "@polkadot/util";
import { subtensor, MultiAddress } from "@polkadot-api/descriptors";
import type { NetworkState } from "../setup.js";
import {
  connectClient,
  createSigner,
  getAccountNonce,
  getBalance,
  waitForFinalizedBlocks,
  sleep,
} from "e2e-shared/client.js";
import { getNextKey, submitEncrypted } from "../helpers.js";

let client: PolkadotClient;
let api: TypedApi<typeof subtensor>;
let state: NetworkState;

const alice = createSigner("//Alice");
const bob = createSigner("//Bob");

beforeAll(async () => {
  const data = await readFile("/tmp/subtensor-e2e/shield/nodes.json", "utf-8");
  state = JSON.parse(data);
  ({ client, api } = await connectClient(state.nodes[0].rpcPort));
});

afterAll(() => {
  client?.destroy();
});

describe("MEV Shield — timing boundaries", () => {
  it("Submit immediately after a new block", async () => {
    // Wait for a fresh finalized block, then immediately read NextKey and submit.
    // This tests the "just after block" boundary where keys just rotated.
    await waitForFinalizedBlocks(client, 1);

    const nextKey = await getNextKey(api);
    expect(nextKey).toBeDefined();

    const balanceBefore = await getBalance(api, bob.address);

    const nonce = await getAccountNonce(api, alice.address);
    const innerTxHex = await api.tx.Balances.transfer_keep_alive({
      dest: MultiAddress.Id(bob.address),
      value: 1_000_000_000n,
    }).sign(alice.signer, { nonce: nonce + 1 });

    await submitEncrypted(api, alice.signer, hexToU8a(innerTxHex), nextKey!, nonce);

    const balanceAfter = await getBalance(api, bob.address);
    expect(balanceAfter).toBeGreaterThan(balanceBefore);
  });

  it("Submit mid-block (~6s after block)", async () => {
    // Wait for a block, then sleep 6s (half of 12s slot) before submitting.
    // The key should still be valid — the same NextKey applies until the next block.
    await waitForFinalizedBlocks(client, 1);
    await sleep(6_000);

    const nextKey = await getNextKey(api);
    expect(nextKey).toBeDefined();

    const balanceBefore = await getBalance(api, bob.address);

    const nonce = await getAccountNonce(api, alice.address);
    const innerTxHex = await api.tx.Balances.transfer_keep_alive({
      dest: MultiAddress.Id(bob.address),
      value: 1_000_000_000n,
    }).sign(alice.signer, { nonce: nonce + 1 });

    await submitEncrypted(api, alice.signer, hexToU8a(innerTxHex), nextKey!, nonce);

    const balanceAfter = await getBalance(api, bob.address);
    expect(balanceAfter).toBeGreaterThan(balanceBefore);
  });

  it("Submit just before next block (~11s after block)", async () => {
    // Wait for a block, then sleep ~11s to submit right before the next slot.
    // The tx enters the pool just as the next block is about to be produced.
    // It should still be included because the N+2 author hasn't changed yet,
    // and PendingKey will match on the next block's proposer check.
    await waitForFinalizedBlocks(client, 1);
    await sleep(11_000);

    const nextKey = await getNextKey(api);
    expect(nextKey).toBeDefined();

    const balanceBefore = await getBalance(api, bob.address);

    const nonce = await getAccountNonce(api, alice.address);
    const innerTxHex = await api.tx.Balances.transfer_keep_alive({
      dest: MultiAddress.Id(bob.address),
      value: 1_000_000_000n,
    }).sign(alice.signer, { nonce: nonce + 1 });

    await submitEncrypted(api, alice.signer, hexToU8a(innerTxHex), nextKey!, nonce);

    const balanceAfter = await getBalance(api, bob.address);
    expect(balanceAfter).toBeGreaterThan(balanceBefore);
  });

  it("Read key, wait full slot (12s), then submit", async () => {
    // Read NextKey, wait a full slot duration, then submit.
    // After one full slot, the key rotates: old NextKey becomes PendingKey.
    // The tx should still be included by the target N+2 author.
    const nextKey = await getNextKey(api);
    expect(nextKey).toBeDefined();

    await sleep(12_000);

    const balanceBefore = await getBalance(api, bob.address);

    const nonce = await getAccountNonce(api, alice.address);
    const innerTxHex = await api.tx.Balances.transfer_keep_alive({
      dest: MultiAddress.Id(bob.address),
      value: 1_000_000_000n,
    }).sign(alice.signer, { nonce: nonce + 1 });

    await submitEncrypted(api, alice.signer, hexToU8a(innerTxHex), nextKey!, nonce);

    const balanceAfter = await getBalance(api, bob.address);
    expect(balanceAfter).toBeGreaterThan(balanceBefore);
  });
});
