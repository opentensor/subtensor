import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { readFile, writeFile, rm } from "node:fs/promises";
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
} from "e2e-shared/client.js";
import { startNode, started, log } from "e2e-shared/node.js";
import { getNextKey, submitEncrypted } from "../helpers.js";

let client: PolkadotClient;
let api: TypedApi<typeof subtensor>;
let state: NetworkState;

const alice = createSigner("//Alice");
const bob = createSigner("//Bob");
const charlie = createSigner("//Charlie");

// Extra nodes join as non-authority full nodes.
const EXTRA_NODE_CONFIGS = [
  { name: "four", port: 30336, rpcPort: 9947, basePath: "/tmp/e2e-shield-four" },
  { name: "five", port: 30337, rpcPort: 9948, basePath: "/tmp/e2e-shield-five" },
  { name: "six", port: 30338, rpcPort: 9949, basePath: "/tmp/e2e-shield-six" },
];

beforeAll(async () => {
  const data = await readFile("/tmp/e2e-shield-nodes.json", "utf-8");
  state = JSON.parse(data);
  ({ client, api } = await connectClient(state.nodes[0].rpcPort));

  // Start 3 additional full nodes to scale from 3 → 6.
  for (const config of EXTRA_NODE_CONFIGS) {
    await rm(config.basePath, { recursive: true, force: true });

    const node = startNode({
      ...config,
      binaryPath: state.binaryPath,
      validator: false,
      chainSpec: state.chainSpec,
    });
    await started(node);
    log(`Extra node ${config.name} started`);

    // Track in state file so global teardown can clean up.
    state.nodes.push({
      name: config.name,
      rpcPort: config.rpcPort,
      port: config.port,
      pid: node.process.pid!,
      basePath: config.basePath,
    });
  }

  // Persist updated state for subsequent test files (edge-cases).
  await writeFile("/tmp/e2e-shield-nodes.json", JSON.stringify(state, null, 2));
});

afterAll(() => {
  client?.destroy();
});

describe("MEV Shield — 6 node scaling", () => {
  it("Network scales to 6 nodes with full peering", async () => {
    expect(state.nodes.length).toBe(6);

    // Verify the network is healthy by checking finalization continues.
    await waitForFinalizedBlocks(client, 2);
  });

  it("Key rotation continues with more peers", async () => {
    const key1 = await getNextKey(api);
    expect(key1).toBeDefined();

    await waitForFinalizedBlocks(client, 2);

    const key2 = await getNextKey(api);
    expect(key2).toBeDefined();
    expect(key2!.length).toBe(1184);
  });

  it("Encrypted tx works with 6 nodes", async () => {
    const nextKey = await getNextKey(api);
    expect(nextKey).toBeDefined();

    const balanceBefore = await getBalance(api, bob.address);

    const nonce = await getAccountNonce(api, alice.address);
    const innerTxHex = await api.tx.Balances.transfer_keep_alive({
      dest: MultiAddress.Id(bob.address),
      value: 5_000_000_000n,
    }).sign(alice.signer, { nonce: nonce + 1 });

    await submitEncrypted(api, alice.signer, hexToU8a(innerTxHex), nextKey!, nonce);

    const balanceAfter = await getBalance(api, bob.address);
    expect(balanceAfter).toBeGreaterThan(balanceBefore);
  });

  it("Multiple encrypted txs in same block with 6 nodes", async () => {
    const nextKey = await getNextKey(api);
    expect(nextKey).toBeDefined();

    const balanceBefore = await getBalance(api, charlie.address);

    const senders = [alice, bob];
    const amount = 1_000_000_000n;
    const txPromises = [];

    for (const sender of senders) {
      const nonce = await getAccountNonce(api, sender.address);

      const innerTxHex = await api.tx.Balances.transfer_keep_alive({
        dest: MultiAddress.Id(charlie.address),
        value: amount,
      }).sign(sender.signer, { nonce: nonce + 1 });

      txPromises.push(
        submitEncrypted(api, sender.signer, hexToU8a(innerTxHex), nextKey!, nonce),
      );
    }

    await Promise.all(txPromises);

    const balanceAfter = await getBalance(api, charlie.address);
    expect(balanceAfter).toBeGreaterThan(balanceBefore);
  });
});
