import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";

describeSuite({
    id: "SMOKE_SUB_STAKING_ADD_STAKING_01",
    title: "Smoke for testing ",
    foundationMethods: "read_only",
    testCases: ({ it, context, log }) => {
        let api: ApiPromise;

        beforeAll(async () => {
            api = context.polkadotJs();
        });

        it({
            id: "T01",
            title: "Test runtime",
            test: async () => {
                const runtimeName = api.runtimeVersion.specName.toString();
                expect(runtimeName).toEqual("node-subtensor");
            },
        });
    },
});
