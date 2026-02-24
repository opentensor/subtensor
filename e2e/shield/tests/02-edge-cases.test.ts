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

describe("MEV Shield — edge cases", () => {
  it("Encrypted tx persists across blocks (CurrentKey fallback)", async () => {
    // The idea: submit an encrypted tx right at a block boundary.
    // Even if the key rotates (NextKey changes), the old key becomes
    // CurrentKey, so the extension still accepts it.
    const nextKey = await getNextKey(api);
    expect(nextKey).toBeDefined();

    const balanceBefore = await getBalance(api, bob.address);

    const nonce = await getAccountNonce(api, alice.address);
    const innerTxHex = await api.tx.Balances.transfer_keep_alive({
      dest: MultiAddress.Id(bob.address),
      value: 2_000_000_000n,
    }).sign(alice.signer, { nonce: nonce + 1 });

    // Submit and wait for finalization — the tx may land in the next block
    // or the one after, where CurrentKey = the old NextKey.
    await submitEncrypted(api, alice.signer, hexToU8a(innerTxHex), nextKey!, nonce);

    const balanceAfter = await getBalance(api, bob.address);
    expect(balanceAfter).toBeGreaterThan(balanceBefore);
  });

  it("Valid ciphertext with invalid inner call", async () => {
    // Encrypt garbage bytes (not a valid extrinsic) using a valid NextKey.
    // The wrapper tx should be included in a block because:
    //   - The ciphertext is well-formed (key_hash, kem_ct, nonce, aead_ct)
    //   - The key_hash matches a known key
    // But the inner decrypted bytes won't decode as a valid extrinsic,
    // so no inner transaction should execute.
    const nextKey = await getNextKey(api);
    expect(nextKey).toBeDefined();

    const balanceBefore = await getBalance(api, bob.address);

    // Garbage "inner transaction" bytes — not a valid extrinsic at all.
    const garbageInner = new Uint8Array(64);
    for (let i = 0; i < 64; i++) garbageInner[i] = (i * 7 + 13) & 0xff;

    const nonce = await getAccountNonce(api, alice.address);

    await submitEncrypted(api, alice.signer, garbageInner, nextKey!, nonce);

    // No balance change — the garbage inner call could not have been a valid transfer.
    const balanceAfter = await getBalance(api, bob.address);
    expect(balanceAfter).toBe(balanceBefore);
  });
});
