import { beforeAll, describeSuite } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";

describeSuite({
    id: "ZOMBIE_SUB_STAKING_ADD_STAKING_01",
    title: "Zombie add staking test suite",
    foundationMethods: "zombie",
    testCases: ({ it, context }) => {
        let polkadotJs: ApiPromise;

        beforeAll(async () => {
            polkadotJs = context.polkadotJs("Node");
        }, 120000);

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
