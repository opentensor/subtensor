import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { checkRuntime, getCurrentKey, getNextKey, waitForFinalizedBlocks } from "../../utils";
import { subtensor } from "@polkadot-api/descriptors";
import type { TypedApi } from "polkadot-api";

describeSuite({
    id: "00.00_basic",
    title: "MEV Shield — key rotation",
    foundationMethods: "zombie",
    testCases: ({ it, context }) => {
        let api: TypedApi<typeof subtensor>;

        beforeAll(async () => {
            api = context.papi("Node").getTypedApi(subtensor);

            await checkRuntime(api);

            await waitForFinalizedBlocks(api, 3);
        }, 120000);

        it({
            id: "T01",
            title: "NextKey and CurrentKey are populated and rotate across blocks",
            test: async () => {
                const nextKey1 = await getNextKey(api);
                expect(nextKey1).toBeDefined();
                expect(nextKey1.length).toBe(1184); // ML-KEM-768 public key

                const currentKey1 = await getCurrentKey(api);
                expect(currentKey1).toBeDefined();
                expect(currentKey1.length).toBe(1184);

                await waitForFinalizedBlocks(api, 2);

                const nextKey2 = await getNextKey(api);
                expect(nextKey2).toBeDefined();
                // Keys should have rotated — nextKey changes each block.
                expect(nextKey2).not.toEqual(nextKey1);

                const currentKey2 = await getCurrentKey(api);
                expect(currentKey2).toBeDefined();
                expect(currentKey2).not.toEqual(currentKey1);
            },
        });

        it({
            id: "T02",
            title: "AuthorKeys stores per-author keys",
            test: async () => {
                const authorities = await api.query.Aura.Authorities.getValue();
                expect(authorities.length).toBeGreaterThan(0);

                let foundKeys = 0;
                for (const authority of authorities) {
                    const key = await api.query.MevShield.AuthorKeys.getValue(authority);
                    if (key) foundKeys++;
                }

                expect(foundKeys).toBeGreaterThan(0);
            },
        });
    },
});
