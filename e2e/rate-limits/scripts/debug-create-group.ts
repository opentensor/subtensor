import { readFile } from "node:fs/promises";
import { Binary, Enum, type TypedApi } from "polkadot-api";
import { subtensor } from "@polkadot-api/descriptors";
import { connectClient } from "e2e-shared/client.js";
import { getAliceSigner } from "e2e-shared/address.js";
import { TX_TIMEOUT, waitForTransactionWithRetry } from "e2e-shared/transactions.js";
import type { NetworkState } from "../setup.js";

const state = JSON.parse(
  await readFile("/tmp/subtensor-e2e/rate-limits/nodes.json", "utf-8"),
) as NetworkState;
const { client, api } = await connectClient(state.nodes[0].rpcPort);

async function dumpState(label: string, api: TypedApi<typeof subtensor>, groupId: number) {
  const [bestNumber, finalizedNumber, nextBest, nextFinalized, groupBest, groupFinalized] =
    await Promise.all([
      api.query.System.Number.getValue({ at: "best" }),
      api.query.System.Number.getValue({ at: "finalized" }),
      api.query.RateLimiting.NextGroupId.getValue({ at: "best" }),
      api.query.RateLimiting.NextGroupId.getValue({ at: "finalized" }),
      api.query.RateLimiting.Groups.getValue(groupId, { at: "best" }),
      api.query.RateLimiting.Groups.getValue(groupId, { at: "finalized" }),
    ]);

  console.log(label, {
    bestNumber,
    finalizedNumber,
    nextBest,
    nextFinalized,
    groupBest,
    groupFinalized,
  });
}

const alice = getAliceSigner();
const groupId = await api.query.RateLimiting.NextGroupId.getValue();
await dumpState("before", api, groupId);
const uniqueName = `debug-create-group-${Date.now()}`;

const internalCall = api.tx.RateLimiting.create_group({
  name: Binary.fromText(uniqueName),
  sharing: Enum("ConfigAndUsage") as never,
});
const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall });

await waitForTransactionWithRetry(api, tx, alice, "debug_create_group");
await dumpState("after_included", api, groupId);

const bestEvents = await api.query.System.Events.getValue({ at: "best" });
const stringify = (value: unknown) =>
  JSON.stringify(
    value,
    (_key, inner) => (typeof inner === "bigint" ? inner.toString() : inner),
    2,
  );

const sudoEvent = [...bestEvents]
  .reverse()
  .find((eventRecord: any) => eventRecord.event?.type === "Sudo");
console.log("sudo_event", stringify(sudoEvent));

const sudoExtrinsicIndex =
  sudoEvent?.phase?.type === "ApplyExtrinsic" ? sudoEvent.phase.value : undefined;
if (sudoExtrinsicIndex !== undefined) {
  const sameExtrinsicEvents = bestEvents.filter(
    (eventRecord: any) =>
      eventRecord.phase?.type === "ApplyExtrinsic" &&
      eventRecord.phase.value === sudoExtrinsicIndex,
  );
  console.log("sudo_extrinsic_events", stringify(sameExtrinsicEvents));
}

const deadline = Date.now() + TX_TIMEOUT;
while (Date.now() < deadline) {
  const groupFinalized = await api.query.RateLimiting.Groups.getValue(groupId, {
    at: "finalized",
  });
  if (groupFinalized !== undefined) {
    break;
  }
  await new Promise((resolve) => setTimeout(resolve, 1_000));
}

await dumpState("after_group_finalized", api, groupId);

client.destroy();
