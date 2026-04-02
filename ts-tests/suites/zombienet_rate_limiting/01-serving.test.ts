import { beforeAll, describeSuite } from "@moonwall/cli";
import { Binary, type TypedApi } from "polkadot-api";
import { subtensor } from "@polkadot-api/descriptors";
import {
    getSignerFromKeypair,
    waitForFinalizedBlocks,
} from "../../utils";
import {
    createRateLimitGroup,
    createRootHotkeyContext,
    expectTransactionFailure,
    groupSharingConfigAndUsage,
    registerCallsInGroup,
    setScopedGroupRateLimit,
    waitForRateLimitTransactionWithRetry,
} from "../../utils/rate-limiting";

describeSuite({
    id: "01_serving",
    title: "Serving rate-limits",
    foundationMethods: "zombie",
    testCases: ({ it, context }) => {
        let api: TypedApi<typeof subtensor>;

        beforeAll(async () => {
            api = context.papi("Node").getTypedApi(subtensor);
        });

        it({
            id: "T01",
            title: "Shares usage between axon variants and keeps prometheus separate",
            test: async () => {
                const ctx = await createRootHotkeyContext(api);
                const signer = getSignerFromKeypair(ctx.hotkey);

                const serveAxon = api.tx.SubtensorModule.serve_axon({
                    netuid: 0,
                    version: 1,
                    ip: 0n,
                    port: 3030,
                    ip_type: 4,
                    protocol: 0,
                    placeholder1: 0,
                    placeholder2: 0,
                });
                const serveAxonTls = api.tx.SubtensorModule.serve_axon_tls({
                    netuid: 0,
                    version: 1,
                    ip: 0n,
                    port: 3030,
                    ip_type: 4,
                    protocol: 0,
                    placeholder1: 0,
                    placeholder2: 0,
                    certificate: Binary.fromBytes(new Uint8Array([1, 2, 3, 4])),
                });
                const servePrometheus = api.tx.SubtensorModule.serve_prometheus({
                    netuid: 0,
                    version: 1,
                    ip: 1_676_056_785n,
                    port: 3031,
                    ip_type: 4,
                });

                const groupId = await createRateLimitGroup(api, "rl-smoke-serving", groupSharingConfigAndUsage());
                await registerCallsInGroup(
                    api,
                    groupId,
                    [serveAxon, serveAxonTls, servePrometheus],
                    "register_smoke_serving_calls"
                );
                await setScopedGroupRateLimit(api, groupId, 0, 2);

                await waitForRateLimitTransactionWithRetry(api, serveAxon, ctx.hotkey, "serve_axon_initial");
                await waitForFinalizedBlocks(api, 1);
                await expectTransactionFailure(serveAxonTls, signer, "serve_axon_tls_rate_limited");

                await waitForRateLimitTransactionWithRetry(api, servePrometheus, ctx.hotkey, "serve_prometheus_initial");
                await waitForFinalizedBlocks(api, 1);
                await expectTransactionFailure(servePrometheus, signer, "serve_prometheus_rate_limited");

                await waitForFinalizedBlocks(api, 1);
                await waitForRateLimitTransactionWithRetry(api, serveAxonTls, ctx.hotkey, "serve_axon_tls_after_window");
            },
        });
    },
});
