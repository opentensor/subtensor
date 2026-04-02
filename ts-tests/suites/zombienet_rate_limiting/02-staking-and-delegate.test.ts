import { beforeAll, describeSuite } from "@moonwall/cli";
import { subtensor } from "@polkadot-api/descriptors";
import type { TypedApi } from "polkadot-api";
import {
    getSignerFromKeypair,
    getStakeRaw,
    tao,
    waitForFinalizedBlocks,
} from "../../utils";
import {
    createOwnedSubnetContext,
    createRateLimitGroup,
    expectTransactionFailure,
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
                const ctx = await createOwnedSubnetContext(api);
                const signer = getSignerFromKeypair(ctx.coldkey);

                const addStake = api.tx.SubtensorModule.add_stake({
                    hotkey: ctx.hotkeyAddress,
                    netuid: ctx.netuid,
                    amount_staked: tao(100),
                });
                const removeStakeTemplate = api.tx.SubtensorModule.remove_stake({
                    hotkey: ctx.hotkeyAddress,
                    netuid: ctx.netuid,
                    amount_unstaked: 1n,
                });

                const groupId = await createRateLimitGroup(api, "rl-smoke-staking", groupSharingConfigAndUsage());
                await registerCallsInGroup(
                    api,
                    groupId,
                    [addStake, removeStakeTemplate],
                    "register_smoke_staking_calls"
                );
                await setGlobalGroupRateLimit(api, groupId, 2);

                await waitForRateLimitTransactionWithRetry(api, addStake, ctx.coldkey, "add_stake_initial");
                await waitForFinalizedBlocks(api, 1);

                const alpha = await getStakeRaw(api, ctx.hotkeyAddress, ctx.coldkeyAddress, ctx.netuid);
                const removeStake = api.tx.SubtensorModule.remove_stake({
                    hotkey: ctx.hotkeyAddress,
                    netuid: ctx.netuid,
                    amount_unstaked: alpha / 2n,
                });

                await expectTransactionFailure(removeStake, signer, "remove_stake_rate_limited");
                await waitForFinalizedBlocks(api, 1);
                await waitForRateLimitTransactionWithRetry(api, removeStake, ctx.coldkey, "remove_stake_after_window");
            },
        });
    },
});
