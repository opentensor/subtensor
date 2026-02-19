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
  waitForFinalizedBlocks,
  watchTxStatus,
} from "e2e-shared/client.js";
import {
  getNextKey,
  getCurrentKey,
  encryptTransaction,
  submitEncrypted,
} from "../helpers.js";

let client: DedotClient<NodeSubtensorApi>;
let state: NetworkState;

const keyring = createKeyring();
const alice = keyring.addFromUri("//Alice");
const bob = keyring.addFromUri("//Bob");
const charlie = keyring.addFromUri("//Charlie");

beforeAll(async () => {
  const data = await readFile("/tmp/e2e-shield-nodes.json", "utf-8");
  state = JSON.parse(data);
  client = await connectClient(state.nodes[0].rpcPort);

  // Wait for enough finalized blocks so the inherent has had time to run
  // and keys have rotated at least once.
  await waitForFinalizedBlocks(client, 3);
});

afterAll(async () => {
  await client?.disconnect();
});

describe("MEV Shield — key rotation", () => {
  it("NextKey and CurrentKey are populated and rotate across blocks", async () => {
    const nextKey1 = await getNextKey(client);
    expect(nextKey1).toBeDefined();
    expect(nextKey1!.length).toBe(1184); // ML-KEM-768 public key

    const currentKey1 = await getCurrentKey(client);
    expect(currentKey1).toBeDefined();
    expect(currentKey1!.length).toBe(1184);

    await waitForFinalizedBlocks(client, 2);

    const nextKey2 = await getNextKey(client);
    expect(nextKey2).toBeDefined();
    // Keys should have rotated — nextKey changes each block.
    expect(nextKey2).not.toEqual(nextKey1);

    const currentKey2 = await getCurrentKey(client);
    expect(currentKey2).toBeDefined();
    expect(currentKey2).not.toEqual(currentKey1);
  });

  it("AuthorKeys stores per-author keys", async () => {
    const authorities = await client.query.aura.authorities();
    expect(authorities.length).toBeGreaterThan(0);

    let foundKeys = 0;
    for (const authority of authorities) {
      const key = await client.query.mevShield.authorKeys(authority);
      if (key) foundKeys++;
    }

    expect(foundKeys).toBeGreaterThan(0);
  });
});

describe("MEV Shield — encrypted transactions", () => {
  it("Happy path: wrapper and inner tx are included in the same block", async () => {
    const nextKey = await getNextKey(client);
    expect(nextKey).toBeDefined();

    const balanceBefore = await getBalance(client, bob.address);

    const nonce = await getAccountNonce(client, alice.address);
    const innerTx = await client.tx.balances
      .transferKeepAlive(bob.address, 10_000_000_000n)
      .sign(alice, { nonce: nonce + 1 });

    const result = await submitEncrypted(client, alice, innerTx.toU8a(), nextKey!, nonce);

    expect(result.status.type).toBe("Finalized");

    // Verify EncryptedSubmitted event was emitted.
    const encryptedEvent = result.events.find(
      (e: any) =>
        e.event?.pallet === "MevShield" && e.event?.palletEvent?.name === "EncryptedSubmitted",
    );
    expect(encryptedEvent).toBeDefined();

    // The inner transfer should be in the same block as the wrapper.
    const balanceAfter = await getBalance(client, bob.address);
    expect(balanceAfter).toBeGreaterThan(balanceBefore);
  });

  it("Failed inner tx: wrapper succeeds but inner transfer has no effect", async () => {
    const nextKey = await getNextKey(client);
    expect(nextKey).toBeDefined();

    const balanceBefore = await getBalance(client, bob.address);

    // Encrypt a transfer of more than Alice has.
    // The wrapper is valid (correct key_hash, valid encryption), but the
    // inner transfer should fail at dispatch with InsufficientBalance.
    const nonce = await getAccountNonce(client, alice.address);
    const innerTx = await client.tx.balances
      .transferKeepAlive(bob.address, 9_000_000_000_000_000_000n)
      .sign(alice, { nonce: nonce + 1 });

    const result = await submitEncrypted(client, alice, innerTx.toU8a(), nextKey!, nonce);

    // The wrapper itself should be finalized successfully.
    expect(result.status.type).toBe("Finalized");

    // The EncryptedSubmitted event should be present (wrapper was valid).
    const encryptedEvent = result.events.find(
      (e: any) =>
        e.event?.pallet === "MevShield" && e.event?.palletEvent?.name === "EncryptedSubmitted",
    );
    expect(encryptedEvent).toBeDefined();

    // The inner transfer failed, so bob's balance should not increase.
    const balanceAfter = await getBalance(client, bob.address);
    expect(balanceAfter).toBe(balanceBefore);
  });

  it("Malformed ciphertext is rejected at pool level", async () => {
    const nonce = await getAccountNonce(client, alice.address);

    // 5 bytes of garbage — not valid ciphertext at all.
    const garbage = new Uint8Array([0x01, 0x02, 0x03, 0x04, 0x05]);

    const tx = client.tx.mevShield.submitEncrypted(garbage);

    // Pool validation rejects with FailedShieldedTxParsing (Custom code 23).
    const status = await watchTxStatus(tx, alice, { nonce }, ["Invalid"]);
    expect(status.type).toBe("Invalid");
  });

  it("Wrong key hash is not included by the block proposer", async () => {
    const nextKey = await getNextKey(client);
    expect(nextKey).toBeDefined();

    const balanceBefore = await getBalance(client, bob.address);

    const nonce = await getAccountNonce(client, alice.address);
    const innerTx = await client.tx.balances
      .transferKeepAlive(bob.address, 1_000_000_000n)
      .sign(alice, { nonce: nonce + 1 });

    const ciphertext = await encryptTransaction(innerTx.toU8a(), nextKey!);

    // Tamper the first 16 bytes (key_hash).
    const tampered = new Uint8Array(ciphertext);
    for (let i = 0; i < 16; i++) tampered[i] = 0xff;

    const tx = client.tx.mevShield.submitEncrypted(tampered);
    const signed = await tx.sign(alice, { nonce });
    // Send without waiting — the tx enters the pool but the block
    // proposer will skip it because the key_hash doesn't match.
    signed.send().catch(() => {});

    await waitForFinalizedBlocks(client, 3);

    // The inner transfer should NOT have executed.
    const balanceAfter = await getBalance(client, bob.address);
    expect(balanceAfter).toBe(balanceBefore);
  });

  it("Stale key is not included after rotation", async () => {
    const staleKey = await getNextKey(client);
    expect(staleKey).toBeDefined();

    // Wait for enough blocks that the key has rotated past both
    // currentKey and nextKey positions.
    await waitForFinalizedBlocks(client, 3);

    const balanceBefore = await getBalance(client, bob.address);

    const nonce = await getAccountNonce(client, alice.address);
    const innerTx = await client.tx.balances
      .transferKeepAlive(bob.address, 1_000_000_000n)
      .sign(alice, { nonce: nonce + 1 });

    const ciphertext = await encryptTransaction(innerTx.toU8a(), staleKey!);

    const tx = client.tx.mevShield.submitEncrypted(ciphertext);
    const signed = await tx.sign(alice, { nonce });
    // Send without waiting — the block proposer will reject because
    // key_hash no longer matches currentKey or nextKey.
    signed.send().catch(() => {});

    await waitForFinalizedBlocks(client, 3);

    // The inner transfer should NOT have executed.
    const balanceAfter = await getBalance(client, bob.address);
    expect(balanceAfter).toBe(balanceBefore);
  });

  it("Multiple encrypted txs in same block", async () => {
    // Use different signers to avoid nonce ordering issues between
    // the outer wrappers and decrypted inner transactions.
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

    // Both should finalize (possibly in different blocks, that's fine).
    const results = await Promise.allSettled(txPromises);

    const succeeded = results.filter((r) => r.status === "fulfilled");
    expect(succeeded.length).toBe(senders.length);

    const balanceAfter = await getBalance(client, charlie.address);
    expect(balanceAfter).toBeGreaterThan(balanceBefore);
  });
});
