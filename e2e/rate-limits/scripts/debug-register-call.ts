import { readFile } from "node:fs/promises";
import { Binary, Enum } from "polkadot-api";
import { subtensor } from "@polkadot-api/descriptors";
import { connectClient } from "e2e-shared/client.js";
import {
  convertPublicKeyToSs58,
  getAliceSigner,
  getRandomSubstrateKeypair,
} from "e2e-shared/address.js";
import {
  waitForFinalizedBlockAdvance,
  waitForSudoTransactionWithRetry,
  waitForTransactionWithRetry,
} from "e2e-shared/transactions.js";
import type { NetworkState } from "../setup.js";

type RpcCapableClient = {
  _request(method: string, params: unknown[]): Promise<unknown>;
};

const state = JSON.parse(
  await readFile("/tmp/subtensor-e2e/rate-limits/nodes.json", "utf-8"),
) as NetworkState;
const { client, api } = await connectClient(state.nodes[0].rpcPort);
const rpcClient = client as typeof client & RpcCapableClient;
const alice = getAliceSigner();
const groupId = await api.query.RateLimiting.NextGroupId.getValue();
const uniqueName = `debug-register-call-${Date.now()}`;

const hotkey = convertPublicKeyToSs58(getRandomSubstrateKeypair().publicKey);
const rootRegister = api.tx.SubtensorModule.root_register({
  hotkey,
});
const rootRegisterEncoded = await rootRegister.getEncodedData();
const normalizedRootRegister = await api.txFromCallData(rootRegisterEncoded);
console.log("rootRegisterEncoded", rootRegisterEncoded.asHex());

const createGroupTx = api.tx.Sudo.sudo({
  call: api.tx.RateLimiting.create_group({
    name: Binary.fromText(uniqueName),
    sharing: Enum("ConfigAndUsage") as never,
  }).decodedCall,
});

await waitForSudoTransactionWithRetry(api, createGroupTx, alice, "debug_create_group");
await waitForFinalizedBlockAdvance(api);

const registerTx = api.tx.Sudo.sudo({
  call: api.tx.RateLimiting.register_call({
    call: normalizedRootRegister.decodedCall as never,
    group: groupId,
  }).decodedCall,
});
const registerCallEncoded = await api.tx.RateLimiting.register_call({
  call: normalizedRootRegister.decodedCall as never,
  group: groupId,
}).getEncodedData();
console.log("registerCallEncoded", registerCallEncoded.asHex());
console.log(
  "registerCallContainsInner",
  registerCallEncoded.asHex().includes(rootRegisterEncoded.asHex().slice(2)),
);

await waitForSudoTransactionWithRetry(api, registerTx, alice, "debug_register_call");
await waitForFinalizedBlockAdvance(api);

const bestEvents = await api.query.System.Events.getValue({ at: "best" });
const callRegistered = [...bestEvents]
  .reverse()
  .find(
    (eventRecord: any) =>
      eventRecord.event?.type === "RateLimiting" &&
      eventRecord.event?.value?.type === "CallRegistered",
  ) as any;
const transaction = callRegistered?.event?.value?.value?.transaction;

const encoder = new TextEncoder();
const rpcResponse = await rpcClient._request("rateLimiting_getRateLimit", [
  Array.from(encoder.encode("SubtensorModule")),
  Array.from(encoder.encode("root_register")),
  null,
]);

const storageGroupBest = transaction
  ? await api.query.RateLimiting.CallGroups.getValue(transaction, { at: "best" })
  : undefined;
const storageGroupFinalized = transaction
  ? await api.query.RateLimiting.CallGroups.getValue(transaction, { at: "finalized" })
  : undefined;

console.log("transaction", transaction);
console.log("storageGroupBest", storageGroupBest);
console.log("storageGroupFinalized", storageGroupFinalized);
console.log(
  "rpcResponse",
  JSON.stringify(rpcResponse, (_key, value) => (typeof value === "bigint" ? value.toString() : value), 2),
);

client.destroy();
