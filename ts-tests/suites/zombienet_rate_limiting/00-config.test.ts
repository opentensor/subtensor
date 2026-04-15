import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { subtensor } from "@polkadot-api/descriptors";
import type { TypedApi } from "polkadot-api";
import {
    generateKeyringPair,
} from "../../utils";
import {
    createRateLimitGroup,
    getCallRateLimit,
    getGroupedResponseGroupId,
    getRateLimitConfig,
    groupSharingConfigAndUsage,
    isGlobalConfig,
    registerCallsInGroup,
    setGlobalGroupRateLimit,
} from "../../utils/rate-limiting";

describeSuite({
    id: "00_config",
    title: "Rate-limits RPC smoke",
    foundationMethods: "zombie",
    testCases: ({ it, context }) => {
        let api: TypedApi<typeof subtensor>;
        let client: any;

        beforeAll(async () => {
            client = context.papi("Node");
            api = client.getTypedApi(subtensor);
        });

        it({
            id: "T01",
            title: "Reports explicit grouped setup created by admin extrinsics",
            test: async () => {
                const hotkey = generateKeyringPair("sr25519").address;
                const newHotkey = generateKeyringPair("sr25519").address;

                const groupId = await createRateLimitGroup(api, "rl-smoke-config", groupSharingConfigAndUsage());
                const swapHotkey = api.tx.SubtensorModule.swap_hotkey({
                    hotkey,
                    new_hotkey: newHotkey,
                    netuid: undefined,
                });

                await registerCallsInGroup(api, groupId, [swapHotkey], "register_smoke_config_calls");
                await setGlobalGroupRateLimit(api, groupId, 3);

                const response = await getCallRateLimit(client, "SubtensorModule", "swap_hotkey");
                expect(response).toBeDefined();
                expect(getGroupedResponseGroupId(response)).toBe(groupId);
                expect(isGlobalConfig(getRateLimitConfig(response))).toBe(true);
            },
        });
    },
});
