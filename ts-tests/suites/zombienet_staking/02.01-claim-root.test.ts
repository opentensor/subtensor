import { expect, beforeAll } from "vitest";
import { describeSuite } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import { getNumRootClaims, sudoSetLockReductionInterval, sudoSetNumRootClaims } from "../../utils";

describeSuite({
    id: "0201_sudo_set_num_root_claims",
    title: "▶ sudo_set_num_root_claims extrinsic",
    foundationMethods: "zombie",
    testCases: ({ it, context, log }) => {
        let api: ApiPromise;

        beforeAll(async () => {
            api = context.polkadotJs("Node");
            await sudoSetLockReductionInterval(api, 1);
        });

        it({
            id: "T0201",
            title: "",
            test: async () => {
                // Get initial value
                const numClaimsBefore = await getNumRootClaims(api);
                log(`Num root claims before: ${numClaimsBefore}`);

                // Set new value (different from current)
                const newValue = numClaimsBefore + 5n;
                await sudoSetNumRootClaims(api, newValue);

                // Verify value changed
                const numClaimsAfter = await getNumRootClaims(api);
                log(`Num root claims after: ${numClaimsAfter}`);

                expect(numClaimsAfter).toBe(newValue);

                log("✅ Successfully set num root claims.");
            },
        });
    },
});
