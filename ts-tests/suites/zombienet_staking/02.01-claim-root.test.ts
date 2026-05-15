import { expect, beforeAll } from "vitest";
import { describeSuite } from "@moonwall/cli";
import { getNumRootClaims, sudoSetLockReductionInterval, sudoSetNumRootClaims } from "../../utils";
import { subtensor } from "@polkadot-api/descriptors";
import type { TypedApi } from "polkadot-api";

describeSuite({
    id: "0201_sudo_set_num_root_claims",
    title: "▶ sudo_set_num_root_claims extrinsic",
    foundationMethods: "zombie",
    testCases: ({ it, context, log }) => {
        let api: TypedApi<typeof subtensor>;

        beforeAll(async () => {
            api = context.papi("Node").getTypedApi(subtensor);
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
