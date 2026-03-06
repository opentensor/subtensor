import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { readFile, writeFile, rm } from "node:fs/promises";
import type { PolkadotClient, TypedApi } from "polkadot-api";
import { Binary } from "polkadot-api";
import { hexToU8a } from "@polkadot/util";
import { subtensor, MultiAddress } from "@polkadot-api/descriptors";
import type { NetworkState } from "../setup.js";
import {
  connectClient,
  createSigner,
  getAccountNonce,
  getBalance,
  sleep,
} from "e2e-shared/client.js";
import { startNode, started, peerCount, stop, log, type Node } from "e2e-shared/node.js";
import { getNextKey, encryptTransaction } from "../helpers.js";

let authorityClient: PolkadotClient;
let authorityApi: TypedApi<typeof subtensor>;
let extraClient: PolkadotClient;
let extraApi: TypedApi<typeof subtensor>;
let state: NetworkState;
let extraNode: Node;

const alice = createSigner("//Alice");
const bob = createSigner("//Bob");

const EXTRA_NODE = {
  name: "mortality-test",
  port: 30339,
  rpcPort: 9950,
  basePath: "/tmp/subtensor-e2e/shield/mortality-test",
};

// MAX_SHIELD_ERA_PERIOD is 8 blocks. With 12s slots, that's ~96s.
const MAX_ERA_BLOCKS = 8;
const SLOT_DURATION_MS = 12_000;
const POLL_INTERVAL_MS = 3_000;

beforeAll(async () => {
  const data = await readFile("/tmp/subtensor-e2e/shield/nodes.json", "utf-8");
  state = JSON.parse(data);

  // Connect to an authority node for key queries.
  ({ client: authorityClient, api: authorityApi } = await connectClient(state.nodes[0].rpcPort));

  // Start a non-authority node to submit txs to.
  await rm(EXTRA_NODE.basePath, { recursive: true, force: true });
  extraNode = startNode({
    ...EXTRA_NODE,
    binaryPath: state.binaryPath,
    validator: false,
    chainSpec: state.chainSpec,
  });
  await started(extraNode);
  await peerCount(extraNode, state.nodes.length);
  log(`Extra non-authority node started for mortality tests`);

  // Track for teardown.
  state.nodes.push({
    ...EXTRA_NODE,
    pid: extraNode.process.pid!,
  });
  await writeFile("/tmp/subtensor-e2e/shield/nodes.json", JSON.stringify(state, null, 2));

  ({ client: extraClient, api: extraApi } = await connectClient(EXTRA_NODE.rpcPort));
});

afterAll(async () => {
  extraClient?.destroy();
  authorityClient?.destroy();
  if (extraNode) {
    try {
      await stop(extraNode);
    } catch {}
  }
});

describe("MEV Shield — mortality eviction", () => {
  it(
    "Tx with tampered key_hash submitted to non-authority is evicted within mortality window",
    async () => {
      // Read a valid NextKey from an authority node, encrypt a real inner tx.
      const nextKey = await getNextKey(authorityApi);
      expect(nextKey).toBeDefined();

      const balanceBefore = await getBalance(extraApi, bob.address);

      const nonce = await getAccountNonce(extraApi, alice.address);
      const innerTxHex = await extraApi.tx.Balances.transfer_keep_alive({
        dest: MultiAddress.Id(bob.address),
        value: 1_000_000_000n,
      }).sign(alice.signer, { nonce: nonce + 1 });

      // Encrypt with valid key, then tamper the key_hash so no proposer will include it.
      const ciphertext = await encryptTransaction(hexToU8a(innerTxHex), nextKey!);
      const tampered = new Uint8Array(ciphertext);
      for (let i = 0; i < 16; i++) tampered[i] = 0xff;

      const tx = extraApi.tx.MevShield.submit_encrypted({
        ciphertext: Binary.fromBytes(tampered),
      });

      // Sign with short mortality (must be ≤ MAX_SHIELD_ERA_PERIOD=8 to pass
      // CheckMortality validation). The tx enters the pool but no proposer
      // will include it (tampered key_hash doesn't match PendingKey).
      const signedHex = await tx.sign(alice.signer, {
        nonce,
        mortality: { mortal: true, period: 8 },
      });

      // Submit via raw RPC to get immediate feedback on pool acceptance.
      let txHash: string;
      try {
        txHash = await extraClient._request(
          "author_submitExtrinsic",
          [signedHex],
        );
        log(`Tx submitted successfully, hash: ${txHash}`);
      } catch (err: unknown) {
        throw new Error(`Tx rejected at pool entry: ${err}`);
      }

      // Verify it's in the pool.
      await sleep(1_000);
      const pending: string[] = await extraClient._request(
        "author_pendingExtrinsics",
        [],
      );
      log(`Pool has ${pending.length} pending tx(s)`);

      // Now poll until the tx disappears (mortality eviction).
      const start = Date.now();
      const maxPollMs = (MAX_ERA_BLOCKS + 4) * SLOT_DURATION_MS;
      let evicted = false;

      log(`Waiting for mortality eviction (up to ${maxPollMs / 1000}s)...`);

      while (Date.now() - start < maxPollMs) {
        await sleep(POLL_INTERVAL_MS);

        const pending: string[] = await extraClient._request(
          "author_pendingExtrinsics",
          [],
        );

        if (pending.length === 0) {
          evicted = true;
          break;
        }
      }

      const elapsed = Date.now() - start;
      log(`Tx ${evicted ? "evicted" : "still in pool"} after ${(elapsed / 1000).toFixed(1)}s`);

      expect(evicted).toBe(true);

      // Eviction should happen within the mortality window plus margin.
      const maxExpectedMs = (MAX_ERA_BLOCKS + 2) * SLOT_DURATION_MS;
      expect(elapsed).toBeLessThan(maxExpectedMs);

      // The inner transfer should NOT have executed.
      const balanceAfter = await getBalance(extraApi, bob.address);
      expect(balanceAfter).toBe(balanceBefore);
    },
    // Longer timeout: wait for mortality window + setup overhead.
    (MAX_ERA_BLOCKS + 8) * SLOT_DURATION_MS,
  );
});
