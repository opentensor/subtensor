import { readFile } from "node:fs/promises";
import type { PolkadotClient, TypedApi } from "polkadot-api";
import { subtensor } from "@polkadot-api/descriptors";
import { connectClient, waitForFinalizedBlocks } from "e2e-shared/client.js";
import { getSignerFromKeypair } from "e2e-shared/address.js";
import { tao } from "e2e-shared/balance.js";
import { getStakeRaw } from "e2e-shared/staking.js";
import {
  createRateLimitGroup,
  createOwnedSubnetContext,
  expectTransactionFailure,
  groupSharingConfigAndUsage,
  registerCallsInGroup,
  setGlobalGroupRateLimit,
} from "e2e-shared/rate-limiting.js";
import { waitForTransactionWithRetry } from "e2e-shared/transactions.js";
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

describe("staking group smoke", () => {
  it("blocks remove_stake immediately after add_stake via shared staking bucket", async () => {
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

    const groupId = await createRateLimitGroup(
      api,
      "rl-smoke-staking",
      groupSharingConfigAndUsage(),
    );
    await registerCallsInGroup(
      api,
      groupId,
      [addStake, removeStakeTemplate],
      "register_smoke_staking_calls",
    );
    await setGlobalGroupRateLimit(api, groupId, 2);

    await waitForTransactionWithRetry(api, addStake, signer, "add_stake_initial");
    await waitForFinalizedBlocks(client, 1);

    const alpha = await getStakeRaw(api, ctx.hotkeyAddress, ctx.coldkeyAddress, ctx.netuid);
    const removeStake = api.tx.SubtensorModule.remove_stake({
      hotkey: ctx.hotkeyAddress,
      netuid: ctx.netuid,
      amount_unstaked: alpha / 2n,
    });

    await expectTransactionFailure(removeStake, signer, "remove_stake_rate_limited");
    await waitForFinalizedBlocks(client, 1);
    await waitForTransactionWithRetry(api, removeStake, signer, "remove_stake_after_window");
  });
});
