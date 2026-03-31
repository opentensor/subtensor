import { readFile } from "node:fs/promises";
import { Binary, type PolkadotClient, type TypedApi } from "polkadot-api";
import { subtensor } from "@polkadot-api/descriptors";
import { connectClient, waitForFinalizedBlocks } from "e2e-shared/client.js";
import { getSignerFromKeypair } from "e2e-shared/address.js";
import {
  createRateLimitGroup,
  createRootHotkeyContext,
  expectTransactionFailure,
  groupSharingConfigAndUsage,
  registerCallsInGroup,
  setScopedGroupRateLimit,
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

describe("serving group", () => {
  it("shares usage between axon variants and keeps prometheus separate", async () => {
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

    const groupId = await createRateLimitGroup(
      api,
      "rl-smoke-serving",
      groupSharingConfigAndUsage(),
    );
    await registerCallsInGroup(
      api,
      groupId,
      [serveAxon, serveAxonTls, servePrometheus],
      "register_smoke_serving_calls",
    );
    await setScopedGroupRateLimit(api, groupId, 0, 2);

    await waitForTransactionWithRetry(api, serveAxon, signer, "serve_axon_initial");
    await waitForFinalizedBlocks(client, 1);
    await expectTransactionFailure(serveAxonTls, signer, "serve_axon_tls_rate_limited");

    await waitForTransactionWithRetry(api, servePrometheus, signer, "serve_prometheus_initial");
    await waitForFinalizedBlocks(client, 1);
    await expectTransactionFailure(servePrometheus, signer, "serve_prometheus_rate_limited");

    await waitForFinalizedBlocks(client, 1);

    await waitForTransactionWithRetry(api, serveAxonTls, signer, "serve_axon_tls_after_window");
  });
});
