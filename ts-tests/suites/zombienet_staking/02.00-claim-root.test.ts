import { expect, beforeAll } from "vitest";
import { describeSuite } from "@moonwall/cli";
import {
    forceSetBalance,
    generateKeyringPair,
    getRootClaimType,
    setRootClaimType,
    sudoSetLockReductionInterval,
} from "../../utils";
import { subtensor } from "@polkadot-api/descriptors";
import type { TypedApi } from "polkadot-api";

describeSuite({
    id: "02_set_root_claim_type",
    title: "▶ set_root_claim_type extrinsic",
    foundationMethods: "zombie",
    testCases: ({ it, context, log }) => {
        let api: TypedApi<typeof subtensor>;

        beforeAll(async () => {
            api = context.papi("Node").getTypedApi(subtensor);
            await sudoSetLockReductionInterval(api, 1);
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
                await setRootClaimType(api, coldkey, { type: "KeepSubnets", subnets: subnetsToKeep });

                // Verify claim type changed
                const claimTypeAfter = await getRootClaimType(api, coldkeyAddress);
                log(`Root claim type after: ${JSON.stringify(claimTypeAfter)}`);

                expect(typeof claimTypeAfter).toBe("object");
                expect((claimTypeAfter as { type: string }).type).toBe("KeepSubnets");
                expect((claimTypeAfter as { subnets: number[] }).subnets).toEqual(subnetsToKeep);

                log("✅ Successfully set root claim type to KeepSubnets.");
            },
        });
    },
});
