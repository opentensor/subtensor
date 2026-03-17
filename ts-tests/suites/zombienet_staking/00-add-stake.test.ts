import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import {
    addNewSubnetwork,
    addStake,
    forceSetBalance,
    generateKeyringPair,
    getStake,
    startCall,
    tao,
} from "../../utils";

describeSuite({
    id: "00_add_stake",
    title: "▶ add_stake extrinsic",
    foundationMethods: "zombie",
    testCases: ({ it, context, log }) => {
        let api: ApiPromise;

        const hotkey = generateKeyringPair("sr25519");
        const coldkey = generateKeyringPair("sr25519");
        const hotkeyAddress = hotkey.address;
        const coldkeyAddress = coldkey.address;
        let netuid: number;

        beforeAll(async () => {
            api = context.polkadotJs("Node");

            await forceSetBalance(api, hotkeyAddress);
            await forceSetBalance(api, coldkeyAddress);
            netuid = await addNewSubnetwork(api, hotkey, coldkey);
            await startCall(api, netuid, coldkey);
        });

        it({
            id: "T01",
            title: "Add staking payable",
            test: async () => {
                // Get initial stake
                const stakeBefore = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);

                // Add stake
                const stakeAmount = tao(100);
                await addStake(api, coldkey, hotkeyAddress, netuid, stakeAmount);

                // Verify stake increased
                const stakeAfter = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
                expect(stakeAfter, "Stake should increase after adding stake").toBeGreaterThan(stakeBefore);

                log("✅ Successfully added stake.");
            },
        });
    },
});
