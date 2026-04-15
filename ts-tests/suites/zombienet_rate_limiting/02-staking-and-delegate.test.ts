import { beforeAll, describeSuite } from "@moonwall/cli";
import { subtensor } from "@polkadot-api/descriptors";
import type { TypedApi } from "polkadot-api";
import {
    addStake,
    sudoSetLockReductionInterval,
    tao,
    waitForBlocks,
} from "../../utils";
import {
    createRateLimitGroup,
    createOwnedSubnetContext,
    expectTransactionFailure,
    getStakeValueForRateLimit,
    groupSharingConfigAndUsage,
    registerCallsInGroup,
    setGlobalGroupRateLimit,
    waitForRateLimitTransactionWithRetry,
} from "../../utils/rate-limiting";

describeSuite({
    id: "02_staking_and_delegate",
    title: "Staking rate-limits",
    foundationMethods: "zombie",
    testCases: ({ it, context }) => {
        let api: TypedApi<typeof subtensor>;

        beforeAll(async () => {
            api = context.papi("Node").getTypedApi(subtensor);
        });

        it({
            id: "T01",
            title: "Blocks remove_stake immediately after add_stake via shared staking bucket",
            test: async () => {
                const rateLimitWindow = 10;
                await sudoSetLockReductionInterval(api, 1);
                const { coldkey, coldkeyAddress, hotkeyAddress, netuid } = await createOwnedSubnetContext(api);

                const addStakeTx = api.tx.SubtensorModule.add_stake({
                    hotkey: hotkeyAddress,
                    netuid,
                    amount_staked: tao(100),
                });
                const removeStakeTemplate = api.tx.SubtensorModule.remove_stake({
                    hotkey: hotkeyAddress,
                    netuid,
                    amount_unstaked: 1n,
                });

                const groupId = await createRateLimitGroup(api, "rl-smoke-staking", groupSharingConfigAndUsage());
                await registerCallsInGroup(
                    api,
                    groupId,
                    [addStakeTx, removeStakeTemplate],
                    "register_smoke_staking_calls"
                );
                await setGlobalGroupRateLimit(api, groupId, rateLimitWindow);

                await addStake(api, coldkey, hotkeyAddress, netuid, tao(200));

                const stakeBeforeRemove = await getStakeValueForRateLimit(api, hotkeyAddress, coldkeyAddress, netuid);
                const removeStake = api.tx.SubtensorModule.remove_stake({
                    hotkey: hotkeyAddress,
                    netuid,
                    amount_unstaked: stakeBeforeRemove,
                });

                await expectTransactionFailure(api, removeStake, coldkey, "remove_stake_rate_limited");
                await waitForBlocks(api, rateLimitWindow);
                const stakeAfterFailedAttempt = await getStakeValueForRateLimit(api, hotkeyAddress, coldkeyAddress, netuid);
                const removeStakeAfterWindow = api.tx.SubtensorModule.remove_stake({
                    hotkey: hotkeyAddress,
                    netuid,
                    amount_unstaked: stakeAfterFailedAttempt,
                });
                await waitForRateLimitTransactionWithRetry(api, removeStakeAfterWindow, coldkey, "remove_stake_after_window");
            },
        });
    },
});
