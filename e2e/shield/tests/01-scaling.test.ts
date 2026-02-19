import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { readFile, writeFile, rm } from "node:fs/promises";
import { DedotClient } from "dedot";
import type { NetworkState } from "../setup.js";
import type { NodeSubtensorApi } from "../../node-subtensor/index.js";
import {
  connectClient,
  createKeyring,
  getAccountNonce,
  getBalance,
  waitForFinalizedBlocks,
} from "e2e-shared/client.js";
import { startNode, started, log } from "e2e-shared/node.js";
import { getNextKey, submitEncrypted } from "../helpers.js";

let client: DedotClient<NodeSubtensorApi>;
let state: NetworkState;

const keyring = createKeyring();
const alice = keyring.addFromUri("//Alice");
const bob = keyring.addFromUri("//Bob");
const charlie = keyring.addFromUri("//Charlie");

// Extra nodes join as non-authority full nodes.
const EXTRA_NODE_CONFIGS = [
  { name: "four", port: 30336, rpcPort: 9947, basePath: "/tmp/e2e-shield-four" },
  { name: "five", port: 30337, rpcPort: 9948, basePath: "/tmp/e2e-shield-five" },
  { name: "six", port: 30338, rpcPort: 9949, basePath: "/tmp/e2e-shield-six" },
];

beforeAll(async () => {
  const data = await readFile("/tmp/e2e-shield-nodes.json", "utf-8");
  state = JSON.parse(data);
  client = await connectClient(state.nodes[0].rpcPort);

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

afterAll(async () => {
  await client?.disconnect();
});

describe("MEV Shield — 6 node scaling", () => {
  it("Network scales to 6 nodes with full peering", async () => {
    expect(state.nodes.length).toBe(6);

    // Verify the network is healthy by checking finalization continues.
    await waitForFinalizedBlocks(client, 2);
  });

  it("Key rotation continues with more peers", async () => {
    const key1 = await getNextKey(client);
    expect(key1).toBeDefined();

    await waitForFinalizedBlocks(client, 2);

    const key2 = await getNextKey(client);
    expect(key2).toBeDefined();
    expect(key2!.length).toBe(1184);
  });

  it("Encrypted tx works with 6 nodes", async () => {
    const nextKey = await getNextKey(client);
    expect(nextKey).toBeDefined();

    const balanceBefore = await getBalance(client, bob.address);

    const nonce = await getAccountNonce(client, alice.address);
    const innerTx = await client.tx.balances
      .transferKeepAlive(bob.address, 5_000_000_000n)
      .sign(alice, { nonce: nonce + 1 });

    const result = await submitEncrypted(client, alice, innerTx.toU8a(), nextKey!, nonce);

    expect(result.status.type).toBe("Finalized");

    const encryptedEvent = result.events.find(
      (e: any) =>
        e.event?.pallet === "MevShield" && e.event?.palletEvent?.name === "EncryptedSubmitted",
    );
    expect(encryptedEvent).toBeDefined();

    const balanceAfter = await getBalance(client, bob.address);
    expect(balanceAfter).toBeGreaterThan(balanceBefore);
  });

  it("Multiple encrypted txs in same block with 6 nodes", async () => {
    const nextKey = await getNextKey(client);
    expect(nextKey).toBeDefined();

    const balanceBefore = await getBalance(client, charlie.address);

    const senders = [alice, bob];
    const amount = 1_000_000_000n;
    const txPromises = [];

    for (const sender of senders) {
      const nonce = await getAccountNonce(client, sender.address);

      const innerTx = await client.tx.balances
        .transferKeepAlive(charlie.address, amount)
        .sign(sender, { nonce: nonce + 1 });

      txPromises.push(submitEncrypted(client, sender, innerTx.toU8a(), nextKey!, nonce));
    }

    const results = await Promise.allSettled(txPromises);

    const succeeded = results.filter((r) => r.status === "fulfilled");
    expect(succeeded.length).toBe(senders.length);

    const balanceAfter = await getBalance(client, charlie.address);
    expect(balanceAfter).toBeGreaterThan(balanceBefore);
  });
});
