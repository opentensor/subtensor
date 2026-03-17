import { expect, beforeAll } from "vitest";
import type { TypedApi } from "polkadot-api";
import { hexToU8a } from "@polkadot/util";
import { subtensor } from "@polkadot-api/descriptors";
import { describeSuite } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import type { KeyringPair } from "@moonwall/util";
import { Keyring } from "@polkadot/keyring";
import { getAccountNonce, getBalance, getNextKey, submitEncrypted, waitForFinalizedBlocks } from "../../utils";

describeSuite({
    id: "02_edge_cases",
    title: "MEV Shield — edge cases",
    foundationMethods: "zombie",
    testCases: ({ it, context }) => {
        let api: ApiPromise;
        let papi: TypedApi<typeof subtensor>;

        let alice: KeyringPair;
        let bob: KeyringPair;

        beforeAll(async () => {
            const keyring = new Keyring({ type: "sr25519" });
            alice = keyring.addFromUri("//Alice");
            bob = keyring.addFromUri("//Bob");

            papi = context.papi("NodePapi").getTypedApi(subtensor);
            api = context.polkadotJs("Node");

            await waitForFinalizedBlocks(api, 2);
        }, 120000);

        it({
            id: "T01",
            title: "Encrypted tx persists across blocks (CurrentKey fallback)",
            test: async () => {
                // The idea: submit an encrypted tx right at a block boundary.
                // Even if the key rotates (NextKey changes), the old key becomes
                // CurrentKey, so the extension still accepts it.
                const nextKey = await getNextKey(api);
                expect(nextKey).toBeDefined();

                const balanceBefore = await getBalance(api, bob.address);

                const nonce = await getAccountNonce(api, alice.address);
                const innerTx = await api.tx.balances
                    .transferKeepAlive(bob.address, 2_000_000_000n)
                    .signAsync(alice, { nonce: nonce + 1 });

                // Submit and wait for finalization — the tx may land in the next block
                // or the one after, where CurrentKey = the old NextKey.
                await submitEncrypted(papi, alice, hexToU8a(innerTx.toHex()), nextKey, nonce);

                const balanceAfter = await getBalance(api, bob.address);
                expect(balanceAfter).toBeGreaterThan(balanceBefore);
            },
        });

        it({
            id: "T02",
            title: "Valid ciphertext with invalid inner call",
            test: async () => {
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

                await submitEncrypted(papi, alice, garbageInner, nextKey, nonce);

                // No balance change — the garbage inner call could not have been a valid transfer.
                const balanceAfter = await getBalance(api, bob.address);
                expect(balanceAfter).toBe(balanceBefore);
            },
        });
    },
});
