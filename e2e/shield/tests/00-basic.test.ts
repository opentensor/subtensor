import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { readFile } from "node:fs/promises";
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
  waitForFinalizedBlocks,
} from "e2e-shared/client.js";
import { getNextKey, getCurrentKey, encryptTransaction, submitEncrypted } from "../helpers.js";

let client: PolkadotClient;
let api: TypedApi<typeof subtensor>;
let state: NetworkState;

const alice = createSigner("//Alice");
const bob = createSigner("//Bob");
const charlie = createSigner("//Charlie");

beforeAll(async () => {
  const data = await readFile("/tmp/subtensor-e2e/shield/nodes.json", "utf-8");
  state = JSON.parse(data);
  ({ client, api } = await connectClient(state.nodes[0].rpcPort));

  // Wait for enough finalized blocks so the inherent has had time to run
  // and keys have rotated at least once.
  await waitForFinalizedBlocks(client, 3);
});

afterAll(() => {
  client?.destroy();
});

describe("MEV Shield — key rotation", () => {
  it("NextKey and CurrentKey are populated and rotate across blocks", async () => {
    const nextKey1 = await getNextKey(api);
    expect(nextKey1).toBeDefined();
    expect(nextKey1!.length).toBe(1184); // ML-KEM-768 public key

    const currentKey1 = await getCurrentKey(api);
    expect(currentKey1).toBeDefined();
    expect(currentKey1!.length).toBe(1184);

    await waitForFinalizedBlocks(client, 2);

    const nextKey2 = await getNextKey(api);
    expect(nextKey2).toBeDefined();
    // Keys should have rotated — nextKey changes each block.
    expect(nextKey2).not.toEqual(nextKey1);

    const currentKey2 = await getCurrentKey(api);
    expect(currentKey2).toBeDefined();
    expect(currentKey2).not.toEqual(currentKey1);
  });

  it("AuthorKeys stores per-author keys", async () => {
    const authorities = await api.query.Aura.Authorities.getValue();
    expect(authorities.length).toBeGreaterThan(0);

    let foundKeys = 0;
    for (const authority of authorities) {
      const key = await api.query.MevShield.AuthorKeys.getValue(authority);
      if (key) foundKeys++;
    }

    expect(foundKeys).toBeGreaterThan(0);
  });
});

describe("MEV Shield — encrypted transactions", () => {
  it("Happy path: wrapper and inner tx are included in the same block", async () => {
    const nextKey = await getNextKey(api);
    expect(nextKey).toBeDefined();

    const balanceBefore = await getBalance(api, bob.address);

    const nonce = await getAccountNonce(api, alice.address);
    const innerTxHex = await api.tx.Balances.transfer_keep_alive({
      dest: MultiAddress.Id(bob.address),
      value: 10_000_000_000n,
    }).sign(alice.signer, { nonce: nonce + 1 });

    await submitEncrypted(api, alice.signer, hexToU8a(innerTxHex), nextKey!, nonce);

    const balanceAfter = await getBalance(api, bob.address);
    expect(balanceAfter).toBeGreaterThan(balanceBefore);
  });

  it("Failed inner tx: wrapper succeeds but inner transfer has no effect", async () => {
    const nextKey = await getNextKey(api);
    expect(nextKey).toBeDefined();

    const balanceBefore = await getBalance(api, bob.address);

    // Encrypt a transfer of more than Alice has.
    // The wrapper is valid (correct key_hash, valid encryption), but the
    // inner transfer should fail at dispatch with InsufficientBalance.
    const nonce = await getAccountNonce(api, alice.address);
    const innerTxHex = await api.tx.Balances.transfer_keep_alive({
      dest: MultiAddress.Id(bob.address),
      value: 9_000_000_000_000_000_000n,
    }).sign(alice.signer, { nonce: nonce + 1 });

    await submitEncrypted(api, alice.signer, hexToU8a(innerTxHex), nextKey!, nonce);

    // The inner transfer failed, so bob's balance should not increase.
    const balanceAfter = await getBalance(api, bob.address);
    expect(balanceAfter).toBe(balanceBefore);
  });

  it("Malformed ciphertext is rejected at pool level", async () => {
    const nonce = await getAccountNonce(api, alice.address);

    // 5 bytes of garbage — not valid ciphertext at all.
    const garbage = new Uint8Array([0x01, 0x02, 0x03, 0x04, 0x05]);

    const tx = api.tx.MevShield.submit_encrypted({
      ciphertext: Binary.fromBytes(garbage),
    });

    // Pool validation rejects with FailedShieldedTxParsing (Custom code 23).
    await expect(tx.signAndSubmit(alice.signer, { nonce })).rejects.toThrow();
  });

  it("Wrong key hash is not included by the block proposer", async () => {
    const nextKey = await getNextKey(api);
    expect(nextKey).toBeDefined();

    const balanceBefore = await getBalance(api, bob.address);

    const nonce = await getAccountNonce(api, alice.address);
    const innerTxHex = await api.tx.Balances.transfer_keep_alive({
      dest: MultiAddress.Id(bob.address),
      value: 1_000_000_000n,
    }).sign(alice.signer, { nonce: nonce + 1 });

    const ciphertext = await encryptTransaction(hexToU8a(innerTxHex), nextKey!);

    // Tamper the first 16 bytes (key_hash).
    const tampered = new Uint8Array(ciphertext);
    for (let i = 0; i < 16; i++) tampered[i] = 0xff;

    const tx = api.tx.MevShield.submit_encrypted({
      ciphertext: Binary.fromBytes(tampered),
    });
    const signedHex = await tx.sign(alice.signer, { nonce });
    // Send without waiting — the tx enters the pool but the block
    // proposer will skip it because the key_hash doesn't match.
    client.submit(signedHex).catch(() => {});

    await waitForFinalizedBlocks(client, 3);

    // The inner transfer should NOT have executed.
    const balanceAfter = await getBalance(api, bob.address);
    expect(balanceAfter).toBe(balanceBefore);
  });

  it("Stale key is not included after rotation", async () => {
    const staleKey = await getNextKey(api);
    expect(staleKey).toBeDefined();

    // Wait for enough blocks that the key has rotated past both
    // currentKey and nextKey positions.
    await waitForFinalizedBlocks(client, 3);

    const balanceBefore = await getBalance(api, bob.address);

    const nonce = await getAccountNonce(api, alice.address);
    const innerTxHex = await api.tx.Balances.transfer_keep_alive({
      dest: MultiAddress.Id(bob.address),
      value: 1_000_000_000n,
    }).sign(alice.signer, { nonce: nonce + 1 });

    const ciphertext = await encryptTransaction(hexToU8a(innerTxHex), staleKey!);

    const tx = api.tx.MevShield.submit_encrypted({
      ciphertext: Binary.fromBytes(ciphertext),
    });
    const signedHex = await tx.sign(alice.signer, { nonce });
    // Send without waiting — the block proposer will reject because
    // key_hash no longer matches currentKey or nextKey.
    client.submit(signedHex).catch(() => {});

    await waitForFinalizedBlocks(client, 3);

    // The inner transfer should NOT have executed.
    const balanceAfter = await getBalance(api, bob.address);
    expect(balanceAfter).toBe(balanceBefore);
  });

  it("Multiple encrypted txs in same block", async () => {
    // Use different signers to avoid nonce ordering issues between
    // the outer wrappers and decrypted inner transactions.
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
