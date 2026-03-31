import { readFile } from "node:fs/promises";
import type { PolkadotClient, TypedApi } from "polkadot-api";
import { subtensor } from "@polkadot-api/descriptors";
import { connectClient } from "e2e-shared/client.js";
import {
  convertPublicKeyToSs58,
  getRandomSubstrateKeypair,
} from "e2e-shared/address.js";
import {
  createRateLimitGroup,
  getCallRateLimit,
  getRateLimitConfig,
  groupSharingConfigAndUsage,
  isGlobalConfig,
  registerCallsInGroup,
  setGlobalGroupRateLimit,
} from "e2e-shared/rate-limiting.js";
import type { NetworkState } from "../setup.js";

let client: PolkadotClient;
let api: TypedApi<typeof subtensor>;

beforeAll(async () => {
  const state = JSON.parse(
    await readFile("/tmp/subtensor-e2e/rate-limits/nodes.json", "utf-8"),
  ) as NetworkState;
  ({ client, api } = await connectClient(state.nodes[0].rpcPort));
});

afterAll(() => {
  client?.destroy();
});

describe("rate-limits rpc smoke", () => {
  it("reports explicit grouped setup created by admin extrinsics", async () => {
    const hotkey = convertPublicKeyToSs58(getRandomSubstrateKeypair().publicKey);
    const newHotkey = convertPublicKeyToSs58(getRandomSubstrateKeypair().publicKey);

    const groupId = await createRateLimitGroup(
      api,
      "rl-smoke-config",
      groupSharingConfigAndUsage(),
    );
    const swapHotkey = api.tx.SubtensorModule.swap_hotkey({
      hotkey,
      new_hotkey: newHotkey,
      netuid: undefined,
    });

    await registerCallsInGroup(api, groupId, [swapHotkey], "register_smoke_config_calls");
    await setGlobalGroupRateLimit(api, groupId, 3);

    const response = await getCallRateLimit(client, "SubtensorModule", "swap_hotkey");
    expect(response).toBeDefined();
    expect(isGlobalConfig(getRateLimitConfig(response))).toBe(true);
  });
});
