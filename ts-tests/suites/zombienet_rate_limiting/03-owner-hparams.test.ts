import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { subtensor } from "@polkadot-api/descriptors";
import type { TypedApi } from "polkadot-api";
import {
    generateKeyringPair,
    sudoSetAdminFreezeWindow,
    sudoSetTempo,
    waitForFinalizedBlocks,
} from "../../utils";
import {
    addNewSubnetworkForRateLimit,
    createRateLimitGroup,
    forceSetBalancesForRateLimit,
    groupSharingConfigOnly,
    registerCallsInGroup,
    setGlobalGroupRateLimit,
    submitTransactionBestEffort,
    startCallForRateLimit,
    waitForRateLimitTransactionWithRetry,
} from "../../utils/rate-limiting";

describeSuite({
    id: "03_owner_hparams",
    title: "Owner hparams rate-limits",
    foundationMethods: "zombie",
    testCases: ({ it, context }) => {
        let api: TypedApi<typeof subtensor>;

        beforeAll(async () => {
            api = context.papi("Node").getTypedApi(subtensor);
        });

        it({
            id: "T01",
            title: "Shares config, keeps usage per hyperparameter, and scopes by netuid",
            test: async () => {
                const coldkey = generateKeyringPair("sr25519");
                const hotkeyA = generateKeyringPair("sr25519");
                const hotkeyB = generateKeyringPair("sr25519");

                await forceSetBalancesForRateLimit(api, [coldkey.address, hotkeyA.address, hotkeyB.address]);

                const netuidA = await addNewSubnetworkForRateLimit(api, hotkeyA, coldkey);
                await startCallForRateLimit(api, netuidA, coldkey);
                const netuidB = await addNewSubnetworkForRateLimit(api, hotkeyB, coldkey);
                await startCallForRateLimit(api, netuidB, coldkey);

                await sudoSetAdminFreezeWindow(api, 0);
                await sudoSetTempo(api, netuidA, 1);
                await sudoSetTempo(api, netuidB, 1);

                const groupId = await createRateLimitGroup(api, "rl-smoke-owner-hparams", groupSharingConfigOnly());
                const cutoffTemplate = api.tx.AdminUtils.sudo_set_activity_cutoff({
                    netuid: netuidA,
                    activity_cutoff: 1,
                });
                const rhoTemplate = api.tx.AdminUtils.sudo_set_rho({
                    netuid: netuidA,
                    rho: 1,
                });
                const burnHalfLifeTemplate = api.tx.AdminUtils.sudo_set_burn_half_life({
                    netuid: netuidA,
                    burn_half_life: 1,
                });
                await registerCallsInGroup(
                    api,
                    groupId,
                    [cutoffTemplate, rhoTemplate, burnHalfLifeTemplate],
                    "register_smoke_owner_hparams_calls"
                );
                await setGlobalGroupRateLimit(api, groupId, 2);

                const currentCutoffA = await api.query.SubtensorModule.ActivityCutoff.getValue(netuidA);
                const currentCutoffB = await api.query.SubtensorModule.ActivityCutoff.getValue(netuidB);
                const currentRhoA = await api.query.SubtensorModule.Rho.getValue(netuidA);
                const cutoffAFirst = api.tx.AdminUtils.sudo_set_activity_cutoff({
                    netuid: netuidA,
                    activity_cutoff: currentCutoffA + 1,
                });
                const cutoffASecond = api.tx.AdminUtils.sudo_set_activity_cutoff({
                    netuid: netuidA,
                    activity_cutoff: currentCutoffA + 2,
                });
                const rhoA = api.tx.AdminUtils.sudo_set_rho({
                    netuid: netuidA,
                    rho: currentRhoA + 1,
                });
                const burnHalfLifeA = api.tx.AdminUtils.sudo_set_burn_half_life({
                    netuid: netuidA,
                    burn_half_life: 361,
                });
                const cutoffB = api.tx.AdminUtils.sudo_set_activity_cutoff({
                    netuid: netuidB,
                    activity_cutoff: currentCutoffB + 1,
                });
                const expectedCutoffAAfterFirst = currentCutoffA + 1;

                await waitForRateLimitTransactionWithRetry(api, cutoffAFirst, coldkey, "owner_cutoff_a_initial");
                await waitForFinalizedBlocks(api, 1);
                await waitForRateLimitTransactionWithRetry(api, rhoA, coldkey, "owner_rho_a_initial");
                await waitForFinalizedBlocks(api, 1);
                await waitForRateLimitTransactionWithRetry(
                    api,
                    burnHalfLifeA,
                    coldkey,
                    "owner_burn_half_life_a_initial"
                );
                await waitForFinalizedBlocks(api, 1);
                await waitForRateLimitTransactionWithRetry(api, cutoffB, coldkey, "owner_cutoff_b_allowed");
                await submitTransactionBestEffort(api, cutoffASecond, coldkey);
                await waitForFinalizedBlocks(api, 2);
                expect(await api.query.SubtensorModule.ActivityCutoff.getValue(netuidA)).toBe(expectedCutoffAAfterFirst);

                await waitForFinalizedBlocks(api, 1);
                await waitForRateLimitTransactionWithRetry(api, cutoffASecond, coldkey, "owner_cutoff_a_after");
            },
        });
    },
});
