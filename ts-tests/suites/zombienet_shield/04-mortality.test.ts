import { expect, beforeAll } from "vitest";
import type { PolkadotClient, TypedApi } from "polkadot-api";
import { Binary } from "polkadot-api";
import { hexToU8a } from "@polkadot/util";
import { subtensor, MultiAddress } from "@polkadot-api/descriptors";
import type { KeyringPair } from "@moonwall/util";
import { Keyring } from "@polkadot/keyring";
import {
    checkRuntime,
    encryptTransaction,
    getAccountNonce,
    getBalance,
    getNextKey,
    getSignerFromKeypair,
    waitForFinalizedBlocks,
} from "../../utils";
import { describeSuite } from "@moonwall/cli";
import { sleep } from "@zombienet/utils";

// MAX_SHIELD_ERA_PERIOD is 8 blocks. With 12s slots, that's ~96s.
const MAX_ERA_BLOCKS = 8;
const SLOT_DURATION_MS = 12_000;
const POLL_INTERVAL_MS = 3_000;

describeSuite({
    id: "04_mortality",
    title: "MEV Shield — mortality eviction",
    foundationMethods: "zombie",
    testCases: ({ it, context, log }) => {
        let apiAuthority: TypedApi<typeof subtensor>;

        let apiFull: TypedApi<typeof subtensor>;
        let clientFull: PolkadotClient;

        let alice: KeyringPair;
        let bob: KeyringPair;

        beforeAll(
            async () => {
                const keyring = new Keyring({ type: "sr25519" });
                alice = keyring.addFromUri("//Alice");
                bob = keyring.addFromUri("//Bob");

                apiAuthority = context.papi("Node").getTypedApi(subtensor);

                clientFull = context.papi("NodeFull");
                apiFull = clientFull.getTypedApi(subtensor);

                await checkRuntime(apiAuthority);

                // Wait for a fresh finalized block, then immediately read NextKey and submit.
                // This tests the "just after block" boundary where keys just rotated.
                await waitForFinalizedBlocks(apiAuthority, 1);
            },
            (MAX_ERA_BLOCKS + 8) * SLOT_DURATION_MS
        );

        it({
            id: "T01",
            title: "Tx with tampered key_hash submitted to non-authority is evicted within mortality window",
            test: async () => {
                // Read a valid NextKey from an authority node, encrypt a real inner tx.
                const nextKey = await getNextKey(apiAuthority);
                expect(nextKey).toBeDefined();

                const balanceBefore = await getBalance(apiFull, bob.address);

                const nonce = await getAccountNonce(apiFull, alice.address);
                const innerTxHex = await apiFull.tx.Balances.transfer_keep_alive({
                    dest: MultiAddress.Id(bob.address),
                    value: 1_000_000_000n,
                }).sign(getSignerFromKeypair(alice), { nonce: nonce + 1 });

                // Encrypt with valid key, then tamper the key_hash so no proposer will include it.
                const ciphertext = await encryptTransaction(hexToU8a(innerTxHex), nextKey);
                const tampered = new Uint8Array(ciphertext);
                for (let i = 0; i < 16; i++) tampered[i] = 0xff;

                const tx = apiFull.tx.MevShield.submit_encrypted({
                    ciphertext: Binary.fromBytes(tampered),
                });

                // Sign with short mortality (must be ≤ MAX_SHIELD_ERA_PERIOD=8 to pass
                // CheckMortality validation). The tx enters the pool but no proposer
                // will include it (tampered key_hash doesn't match PendingKey).
                const signedHex = await tx.sign(getSignerFromKeypair(alice), {
                    nonce,
                    mortality: { mortal: true, period: 8 },
                });

                // Submit via raw RPC to get immediate feedback on pool acceptance.
                let txHash: string;
                try {
                    txHash = await clientFull._request("author_submitExtrinsic", [signedHex]);
                    log(`Tx submitted successfully, hash: ${txHash}`);
                } catch (err: unknown) {
                    throw new Error(`Tx rejected at pool entry: ${err}`);
                }

                // Verify it's in the pool.
                await sleep(1_000);
                const pending: string[] = await clientFull._request("author_pendingExtrinsics", []);
                log(`Pool has ${pending.length} pending tx(s)`);

                // Now poll until the tx disappears (mortality eviction).
                const start = Date.now();
                const maxPollMs = (MAX_ERA_BLOCKS + 4) * SLOT_DURATION_MS;
                let evicted = false;

                log(`Waiting for mortality eviction (up to ${maxPollMs / 1000}s)...`);

                while (Date.now() - start < maxPollMs) {
                    await sleep(POLL_INTERVAL_MS);

                    const pending: string[] = await clientFull._request("author_pendingExtrinsics", []);

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
                const balanceAfter = await getBalance(apiFull, bob.address);
                expect(balanceAfter).toBe(balanceBefore);
            },
        });
    },
});
