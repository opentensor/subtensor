import { Binary, Enum, type PolkadotClient, type TypedApi } from "polkadot-api";
import type { PolkadotSigner } from "polkadot-api/signer";
import { subtensor } from "@polkadot-api/descriptors";
import { KeyPair } from "@polkadot-labs/hdkd-helpers";
import {
  convertPublicKeyToSs58,
  getRandomSubstrateKeypair,
  getSignerFromKeypair,
  getAliceSigner,
} from "./address.js";
import { forceSetBalances } from "./balance.js";
import { addNewSubnetwork, startCall } from "./subnet.js";
import {
  TX_TIMEOUT,
  waitForFinalizedBlockAdvance,
  waitForSudoTransactionWithRetry,
  waitForTransactionWithRetry,
} from "./transactions.js";

export const GROUP_SERVE = 0;
export const GROUP_DELEGATE_TAKE = 1;
export const GROUP_WEIGHTS_SET = 2;
export const GROUP_REGISTER_NETWORK = 3;
export const GROUP_OWNER_HPARAMS = 4;
export const GROUP_STAKING_OPS = 5;
export const GROUP_SWAP_KEYS = 6;

type RpcCapableClient = PolkadotClient & {
  _request(method: string, params: unknown[]): Promise<unknown>;
};

export const rateLimitTargetGroup = (groupId: number) => Enum("Group", groupId);

export const rateLimitKindExact = (limit: bigint | number) =>
  Enum("Exact", typeof limit === "bigint" ? Number(limit) : limit);

export const groupSharingConfigAndUsage = () => Enum("ConfigAndUsage");

export const groupSharingConfigOnly = () => Enum("ConfigOnly");

async function waitForGroupAtFinalized(
  api: TypedApi<typeof subtensor>,
  groupId: number,
  timeoutMs = TX_TIMEOUT,
): Promise<void> {
  const deadline = Date.now() + timeoutMs;

  while (Date.now() < deadline) {
    const group = await api.query.RateLimiting.Groups.getValue(groupId, { at: "finalized" });
    if (group !== undefined) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 1_000));
  }

  throw new Error(`Timed out waiting for group ${groupId} at finalized`);
}

export async function createRateLimitGroup(
  api: TypedApi<typeof subtensor>,
  name: string,
  sharing: any,
): Promise<number> {
  const alice = getAliceSigner();
  const groupId = await api.query.RateLimiting.NextGroupId.getValue();
  const internalCall = api.tx.RateLimiting.create_group({
    name: Binary.fromText(name),
    sharing: sharing as never,
  });
  const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall });
  await waitForSudoTransactionWithRetry(api, tx, alice, `create_group_${name}`);
  await waitForGroupAtFinalized(api, groupId);
  return groupId;
}

export async function registerCallsInGroup(
  api: TypedApi<typeof subtensor>,
  groupId: number,
  calls: { decodedCall: unknown }[],
  label: string,
): Promise<void> {
  const alice = getAliceSigner();
  const internalCalls = calls.map((call) =>
    api.tx.RateLimiting.register_call({
      call: call.decodedCall as never,
      group: groupId,
    }).decodedCall,
  );
  const batch = api.tx.Utility.batch_all({ calls: internalCalls });
  const tx = api.tx.Sudo.sudo({ call: batch.decodedCall });
  await waitForSudoTransactionWithRetry(api, tx, alice, label);
  await waitForFinalizedBlockAdvance(api);
}

export async function setGlobalGroupRateLimit(
  api: TypedApi<typeof subtensor>,
  groupId: number,
  limit: bigint | number,
): Promise<void> {
  const target = rateLimitTargetGroup(groupId);
  const current = await api.query.RateLimiting.Limits.getValue(target as never);
  const currentValue =
    current && (current as any).type === "Global" && (current as any).value?.type === "Exact"
      ? BigInt((current as any).value.value)
      : undefined;
  const nextValue = BigInt(limit);
  if (currentValue === nextValue) {
    return;
  }

  const alice = getAliceSigner();
  const internalCall = api.tx.RateLimiting.set_rate_limit({
    target: target as never,
    scope: undefined,
    limit: rateLimitKindExact(limit) as never,
  });
  const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall });
  await waitForSudoTransactionWithRetry(api, tx, alice, `set_group_rate_limit_${groupId}`);
  await waitForFinalizedBlockAdvance(api);
}

export async function setScopedGroupRateLimit(
  api: TypedApi<typeof subtensor>,
  groupId: number,
  scope: number,
  limit: bigint | number,
): Promise<void> {
  const target = rateLimitTargetGroup(groupId);
  const current = await api.query.RateLimiting.Limits.getValue(target as never);
  const entries =
    current && (current as any).type === "Scoped" ? Array.from((current as any).value as any[]) : [];
  const existing = entries.find((entry: any) => Number(entry[0]) === scope);
  const currentValue = existing ? BigInt(existing[1].value) : undefined;
  const nextValue = BigInt(limit);
  if (currentValue === nextValue) {
    return;
  }

  const alice = getAliceSigner();
  const internalCall = api.tx.RateLimiting.set_rate_limit({
    target: target as never,
    scope,
    limit: rateLimitKindExact(limit) as never,
  });
  const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall });
  await waitForSudoTransactionWithRetry(
    api,
    tx,
    alice,
    `set_scoped_group_rate_limit_${groupId}`,
  );
  await waitForFinalizedBlockAdvance(api);
}

export async function getCallRateLimit(
  client: PolkadotClient,
  pallet: string,
  extrinsic: string,
): Promise<any> {
  const rpcClient = client as RpcCapableClient;
  const encoder = new TextEncoder();
  return rpcClient._request("rateLimiting_getRateLimit", [
    Array.from(encoder.encode(pallet)),
    Array.from(encoder.encode(extrinsic)),
    null,
  ]);
}

export function getGroupedResponseGroupId(response: any): number | undefined {
  if (response && typeof response === "object" && "group_id" in response) {
    return Number((response as any).group_id);
  }
  if (response?.type === "grouped") {
    return Number(response.value?.group_id);
  }
  if (response?.type === "Grouped") {
    return Number(response.value?.group_id);
  }
  if (response && typeof response === "object" && "Grouped" in response) {
    return Number((response as any).Grouped.group_id);
  }
  if (response && typeof response === "object") {
    const [key, value] = Object.entries(response)[0] ?? [];
    if (typeof key === "string" && key.toLowerCase() === "grouped") {
      return Number((value as any)?.group_id);
    }
  }
  return undefined;
}

export function getRateLimitConfig(response: any): any {
  if (response && typeof response === "object") {
    if ("group_id" in response && "limit" in response) {
      return (response as any).limit;
    }
    if (response.type === "grouped") {
      return response.value?.limit;
    }
    if (response.type === "standalone") {
      return response.value?.limit;
    }
    if (response.type === "Grouped") {
      return response.value?.limit;
    }
    if (response.type === "Standalone") {
      return response.value?.limit;
    }
    if ("Grouped" in response) {
      return (response as any).Grouped.limit;
    }
    if ("Standalone" in response) {
      return (response as any).Standalone.limit;
    }
    const [key, value] = Object.entries(response)[0] ?? [];
    if (typeof key === "string") {
      if (key.toLowerCase() === "grouped") {
        return (value as any)?.limit;
      }
      if (key.toLowerCase() === "standalone") {
        return (value as any)?.limit;
      }
    }
  }
  return undefined;
}

export function isScopedConfig(config: any): boolean {
  return Boolean(
    config &&
      ((typeof config === "object" && "Scoped" in config) || config.type === "Scoped"),
  );
}

export function isGlobalConfig(config: any): boolean {
  return Boolean(
    config &&
      ((typeof config === "object" && "Global" in config) || config.type === "Global"),
  );
}

export async function rootRegister(
  api: TypedApi<typeof subtensor>,
  coldkey: KeyPair,
  hotkeyAddress: string,
): Promise<void> {
  const signer = getSignerFromKeypair(coldkey);
  const tx = api.tx.SubtensorModule.root_register({ hotkey: hotkeyAddress });
  await waitForTransactionWithRetry(api, tx, signer, "root_register");
}

export type RootHotkeyContext = {
  coldkey: KeyPair;
  hotkey: KeyPair;
  coldkeyAddress: string;
  hotkeyAddress: string;
};

export async function createRootHotkeyContext(
  api: TypedApi<typeof subtensor>,
): Promise<RootHotkeyContext> {
  const coldkey = getRandomSubstrateKeypair();
  const hotkey = getRandomSubstrateKeypair();
  const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);
  const hotkeyAddress = convertPublicKeyToSs58(hotkey.publicKey);

  await forceSetBalances(api, [coldkeyAddress, hotkeyAddress]);
  await rootRegister(api, coldkey, hotkeyAddress);

  return { coldkey, hotkey, coldkeyAddress, hotkeyAddress };
}

export type OwnedSubnetContext = {
  coldkey: KeyPair;
  hotkey: KeyPair;
  coldkeyAddress: string;
  hotkeyAddress: string;
  netuid: number;
};

export async function createOwnedSubnetContext(
  api: TypedApi<typeof subtensor>,
): Promise<OwnedSubnetContext> {
  const coldkey = getRandomSubstrateKeypair();
  const hotkey = getRandomSubstrateKeypair();
  const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);
  const hotkeyAddress = convertPublicKeyToSs58(hotkey.publicKey);

  await forceSetBalances(api, [coldkeyAddress, hotkeyAddress]);

  const netuid = await addNewSubnetwork(api, hotkey, coldkey);
  await startCall(api, netuid, coldkey);

  return { coldkey, hotkey, coldkeyAddress, hotkeyAddress, netuid };
}

export async function expectTransactionFailure(
  tx: any,
  signer: PolkadotSigner,
  label: string,
  timeoutMs = 20_000,
): Promise<unknown> {
  return new Promise((resolve, reject) => {
    let settled = false;
    const finish = (cb: () => void) => {
      if (settled) return;
      settled = true;
      clearTimeout(timeoutId);
      cb();
    };

    const subscription = tx.signSubmitAndWatch(signer).subscribe({
      next(value: any) {
        if (value.type === "txBestBlocksState" && value.found) {
          subscription.unsubscribe();
          if (value.ok) {
            finish(() =>
              reject(new Error(`[${label}] succeeded unexpectedly with tx ${value.txHash}`)),
            );
          } else {
            finish(() => resolve(value.dispatchError));
          }
        } else if (value.type === "txBestBlocksState" && value.isValid === false) {
          subscription.unsubscribe();
          finish(() => resolve(new Error(`[${label}] transaction rejected before inclusion`)));
        }
      },
      error(error: unknown) {
        subscription.unsubscribe();
        finish(() => resolve(error));
      },
    });

    const timeoutId = setTimeout(() => {
      subscription.unsubscribe();
      finish(() => reject(new Error(`[${label}] timed out waiting for failure`)));
    }, timeoutMs);
  });
}
