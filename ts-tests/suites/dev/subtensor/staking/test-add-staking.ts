import { beforeAll, describeSuite } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";

describeSuite({
    id: "DEV_SUB_STAKING_ADD_STAKING_01",
    title: "Add staking test suite",
    foundationMethods: "dev",
    testCases: ({ it, context }) => {
        let polkadotJs: ApiPromise;

        beforeAll(() => {
            polkadotJs = context.polkadotJs();
        });

        it({
            id: "T01",
            title: "Add staking payable",
            test: async () => {
                const runtimeName = polkadotJs.runtimeVersion.specName.toString();
                console.log("runtimeName", runtimeName);
            },
        });
    },
});
