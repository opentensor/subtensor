import { beforeAll, describeSuite } from "@moonwall/cli";
import { subtensor } from "@polkadot-api/descriptors";
import type { TypedApi } from "polkadot-api";
import {
    generateKeyringPair,
    getSignerFromKeypair,
    sudoSetAdminFreezeWindow,
    sudoSetTempo,
    waitForFinalizedBlocks,
} from "../../utils";
import {
    addNewSubnetworkForRateLimit,
    createRateLimitGroup,
    expectTransactionFailure,
    forceSetBalancesForRateLimit,
    groupSharingConfigOnly,
    registerCallsInGroup,
    setGlobalGroupRateLimit,
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
                const ownerSigner = getSignerFromKeypair(coldkey);

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
                await registerCallsInGroup(
                    api,
                    groupId,
                    [cutoffTemplate, rhoTemplate],
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
                const cutoffB = api.tx.AdminUtils.sudo_set_activity_cutoff({
                    netuid: netuidB,
                    activity_cutoff: currentCutoffB + 1,
                });

                await waitForRateLimitTransactionWithRetry(api, cutoffAFirst, coldkey, "owner_cutoff_a_initial");
                await waitForFinalizedBlocks(api, 1);
                await waitForRateLimitTransactionWithRetry(api, rhoA, coldkey, "owner_rho_a_initial");
                await waitForRateLimitTransactionWithRetry(api, cutoffB, coldkey, "owner_cutoff_b_allowed");
                await expectTransactionFailure(cutoffASecond, ownerSigner, "owner_cutoff_a_rate_limited");

                await waitForFinalizedBlocks(api, 1);
                await waitForRateLimitTransactionWithRetry(api, cutoffASecond, coldkey, "owner_cutoff_a_after");
            },
        });
    },
});
