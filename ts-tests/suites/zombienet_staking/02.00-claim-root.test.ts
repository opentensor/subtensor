import { expect, beforeAll } from "vitest";
import { describeSuite } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import {
    forceSetBalance,
    generateKeyringPair,
    getRootClaimType,
    type KeepSubnetType,
    setRootClaimType,
} from "../../utils";

describeSuite({
    id: "02_set_root_claim_type",
    title: "▶ set_root_claim_type extrinsic",
    foundationMethods: "zombie",
    testCases: ({ it, context, log }) => {
        let api: ApiPromise;

        beforeAll(async () => {
            api = context.polkadotJs("Node");
        });

        it({
            id: "T0101",
            title: "should set root claim type to Keep",
            test: async () => {
                const coldkey = generateKeyringPair("sr25519");
                const coldkeyAddress = coldkey.address;

                await forceSetBalance(api, coldkeyAddress);

                // Check initial claim type (default is "Swap")
                const claimTypeBefore = await getRootClaimType(api, coldkeyAddress);
                log(`Root claim type before: ${claimTypeBefore}`);

                // Set root claim type to Keep
                await setRootClaimType(api, coldkey, "Keep");

                // Verify claim type changed
                const claimTypeAfter = await getRootClaimType(api, coldkeyAddress);
                log(`Root claim type after: ${claimTypeAfter}`);

                expect(claimTypeAfter).toBe("Keep");

                log("✅ Successfully set root claim type to Keep.");
            },
        });

        it({
            id: "T0102",
            title: "should set root claim type to Swap",
            test: async () => {
                const coldkey = generateKeyringPair("sr25519");
                const coldkeyAddress = coldkey.address;

                await forceSetBalance(api, coldkeyAddress);

                // First set to Keep so we can verify the change to Swap
                await setRootClaimType(api, coldkey, "Keep");
                const claimTypeBefore = await getRootClaimType(api, coldkeyAddress);
                log(`Root claim type before: ${claimTypeBefore}`);
                expect(claimTypeBefore).toBe("Keep");

                // Set root claim type to Swap
                await setRootClaimType(api, coldkey, "Swap");

                // Verify claim type changed
                const claimTypeAfter = await getRootClaimType(api, coldkeyAddress);
                log(`Root claim type after: ${claimTypeAfter}`);

                expect(claimTypeAfter).toBe("Swap");

                log("✅ Successfully set root claim type to Swap.");
            },
        });

        it({
            id: "T0103",
            title: "should set root claim type to KeepSubnets",
            test: async () => {
                const coldkey = generateKeyringPair("sr25519");
                const coldkeyAddress = coldkey.address;

                await forceSetBalance(api, coldkeyAddress);

                // Check initial claim type (default is "Swap")
                const claimTypeBefore = await getRootClaimType(api, coldkeyAddress);
                log(`Root claim type before: ${JSON.stringify(claimTypeBefore)}`);

                // Set root claim type to KeepSubnets with specific subnets
                const subnetsToKeep = [1, 2];
                await setRootClaimType(api, coldkey, { KeepSubnets: { subnets: subnetsToKeep } });

                // Verify claim type changed
                const claimTypeAfter = await getRootClaimType(api, coldkeyAddress);
                log(`Root claim type after: ${JSON.stringify(claimTypeAfter)}`);

                expect(typeof claimTypeAfter).toBe("object");
                expect(!!(claimTypeAfter as KeepSubnetType).KeepSubnets).toBe(true);
                expect((claimTypeAfter as KeepSubnetType).KeepSubnets.subnets).toEqual(subnetsToKeep);

                log("✅ Successfully set root claim type to KeepSubnets.");
            },
        });
    },
});
