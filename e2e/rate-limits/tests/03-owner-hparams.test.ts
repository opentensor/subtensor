import { readFile } from "node:fs/promises";
import type { PolkadotClient, TypedApi } from "polkadot-api";
import { subtensor } from "@polkadot-api/descriptors";
import { connectClient, waitForFinalizedBlocks } from "e2e-shared/client.js";
import {
  convertPublicKeyToSs58,
  getRandomSubstrateKeypair,
  getSignerFromKeypair,
} from "e2e-shared/address.js";
import { forceSetBalances } from "e2e-shared/balance.js";
import {
  createRateLimitGroup,
  expectTransactionFailure,
  groupSharingConfigOnly,
  registerCallsInGroup,
  setGlobalGroupRateLimit,
} from "e2e-shared/rate-limiting.js";
import { addNewSubnetwork, startCall } from "e2e-shared/subnet.js";
import {
  sudoSetAdminFreezeWindow,
  sudoSetTempo,
} from "e2e-shared/staking.js";
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

describe("owner hparams group", () => {
  it("shares config, keeps usage per hyperparameter, and scopes by netuid", async () => {
    const coldkey = getRandomSubstrateKeypair();
    const hotkeyA = getRandomSubstrateKeypair();
    const hotkeyB = getRandomSubstrateKeypair();
    const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);
    const hotkeyAddressA = convertPublicKeyToSs58(hotkeyA.publicKey);
    const hotkeyAddressB = convertPublicKeyToSs58(hotkeyB.publicKey);
    const ownerSigner = getSignerFromKeypair(coldkey);

    await forceSetBalances(api, [coldkeyAddress, hotkeyAddressA, hotkeyAddressB]);

    const netuidA = await addNewSubnetwork(api, hotkeyA, coldkey);
    await startCall(api, netuidA, coldkey);
    const netuidB = await addNewSubnetwork(api, hotkeyB, coldkey);
    await startCall(api, netuidB, coldkey);

    await sudoSetAdminFreezeWindow(api, 0);
    await sudoSetTempo(api, netuidA, 1);
    await sudoSetTempo(api, netuidB, 1);

    const groupId = await createRateLimitGroup(
      api,
      "rl-smoke-owner-hparams",
      groupSharingConfigOnly(),
    );
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
      "register_smoke_owner_hparams_calls",
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

    await waitForTransactionWithRetry(api, cutoffAFirst, ownerSigner, "owner_cutoff_a_initial");
    await waitForFinalizedBlocks(client, 1);
    await waitForTransactionWithRetry(api, rhoA, ownerSigner, "owner_rho_a_initial");
    await waitForTransactionWithRetry(api, cutoffB, ownerSigner, "owner_cutoff_b_allowed");
    await expectTransactionFailure(cutoffASecond, ownerSigner, "owner_cutoff_a_rate_limited");

    await waitForFinalizedBlocks(client, 1);
    await waitForTransactionWithRetry(api, cutoffASecond, ownerSigner, "owner_cutoff_a_after");
  });
});
