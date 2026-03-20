import { expect, beforeAll } from "vitest";
import { describeSuite } from "@moonwall/cli";
import {
    addNewSubnetwork,
    forceSetBalance,
    generateKeyringPair,
    getRootClaimThreshold,
    startCall,
    sudoSetLockReductionInterval,
    sudoSetRootClaimThreshold,
} from "../../utils";
import { subtensor } from "@polkadot-api/descriptors";
import type { TypedApi } from "polkadot-api";

describeSuite({
    id: "0202_sudo_set_root_claim_threshold",
    title: "▶ sudo_set_root_claim_threshold extrinsic",
    foundationMethods: "zombie",
    testCases: ({ it, context, log }) => {
        let api: TypedApi<typeof subtensor>;

        beforeAll(async () => {
            api = context.papi("Node").getTypedApi(subtensor);
            await sudoSetLockReductionInterval(api, 1);
        });

        it({
            id: "T0301",
            title: "should set root claim threshold for subnet",
            test: async () => {
                // Create a subnet to test with
                const hotkey = generateKeyringPair("sr25519");
                const coldkey = generateKeyringPair("sr25519");
                const hotkeyAddress = hotkey.address;
                const coldkeyAddress = coldkey.address;

                await forceSetBalance(api, hotkeyAddress);
                await forceSetBalance(api, coldkeyAddress);

                const netuid = await addNewSubnetwork(api, hotkey, coldkey);
                await startCall(api, netuid, coldkey);

                // Get initial threshold
                const thresholdBefore = await getRootClaimThreshold(api, netuid);
                log(`Root claim threshold before: ${thresholdBefore}`);

                // Set new threshold value (MAX_ROOT_CLAIM_THRESHOLD is 10_000_000)
                // The value is stored as I96F32 fixed-point with 32 fractional bits
                const newThreshold = 1_000_000n;
                await sudoSetRootClaimThreshold(api, netuid, newThreshold);

                // Verify threshold changed
                // I96F32 encoding: newThreshold * 2^32 = 1_000_000 * 4294967296 = 4294967296000000
                const thresholdAfter = await getRootClaimThreshold(api, netuid);
                log(`Root claim threshold after: ${thresholdAfter}`);

                const expectedStoredValue = newThreshold * (1n << 32n); // I96F32 encoding
                expect(thresholdAfter).toBe(expectedStoredValue);

                log("✅ Successfully set root claim threshold.");
            },
        });
    },
});
