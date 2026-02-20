import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { readFile } from "node:fs/promises";
import { DedotClient } from "dedot";
import type { NetworkState } from "../setup.js";
import type { NodeSubtensorApi } from "../../node-subtensor/index.js";
import {
  connectClient,
  createKeyring,
  getAccountNonce,
  getBalance,
} from "e2e-shared/client.js";
import { getNextKey, submitEncrypted } from "../helpers.js";
let client: DedotClient<NodeSubtensorApi>;
let state: NetworkState;

const keyring = createKeyring();
const alice = keyring.addFromUri("//Alice");
const bob = keyring.addFromUri("//Bob");

beforeAll(async () => {
  const data = await readFile("/tmp/e2e-shield-nodes.json", "utf-8");
  state = JSON.parse(data);
  client = await connectClient(state.nodes[0].rpcPort);
});

afterAll(async () => {
  await client?.disconnect();
});

describe("MEV Shield — edge cases", () => {
  it("Encrypted tx persists across blocks (CurrentKey fallback)", async () => {
    // The idea: submit an encrypted tx right at a block boundary.
    // Even if the key rotates (NextKey changes), the old key becomes
    // CurrentKey, so the extension still accepts it.
    const nextKey = await getNextKey(client);
    expect(nextKey).toBeDefined();

    const balanceBefore = await getBalance(client, bob.address);

    const nonce = await getAccountNonce(client, alice.address);
    const innerTx = await client.tx.balances
      .transferKeepAlive(bob.address, 2_000_000_000n)
      .sign(alice, { nonce: nonce + 1 });

    // Submit and wait for finalization — the tx may land in the next block
    // or the one after, where CurrentKey = the old NextKey.
    const result = await submitEncrypted(client, alice, innerTx.toU8a(), nextKey!, nonce);

    expect(result.status.type).toBe("Finalized");

    const balanceAfter = await getBalance(client, bob.address);
    expect(balanceAfter).toBeGreaterThan(balanceBefore);
  });

  it("Valid ciphertext with invalid inner call", async () => {
    // Encrypt garbage bytes (not a valid extrinsic) using a valid NextKey.
    // The wrapper tx should be included in a block because:
    //   - The ciphertext is well-formed (key_hash, kem_ct, nonce, aead_ct)
    //   - The key_hash matches a known key
    // But the inner decrypted bytes won't decode as a valid extrinsic,
    // so no inner transaction should execute.
    const nextKey = await getNextKey(client);
    expect(nextKey).toBeDefined();

    const balanceBefore = await getBalance(client, bob.address);

    // Garbage "inner transaction" bytes — not a valid extrinsic at all.
    const garbageInner = new Uint8Array(64);
    for (let i = 0; i < 64; i++) garbageInner[i] = (i * 7 + 13) & 0xff;

    const nonce = await getAccountNonce(client, alice.address);

    const result = await submitEncrypted(client, alice, garbageInner, nextKey!, nonce);

    // The wrapper should be finalized successfully.
    expect(result.status.type).toBe("Finalized");

    // The EncryptedSubmitted event should be emitted for the wrapper.
    const encryptedEvent = result.events.find(
      (e: any) =>
        e.event?.pallet === "MevShield" && e.event?.palletEvent?.name === "EncryptedSubmitted",
    );
    expect(encryptedEvent).toBeDefined();

    // No balance change — the garbage inner call could not have been a valid transfer.
    const balanceAfter = await getBalance(client, bob.address);
    expect(balanceAfter).toBe(balanceBefore);
  });
});
