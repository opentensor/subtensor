import { Binary, Enum, type TypedApi } from "polkadot-api";
import type { PolkadotSigner } from "polkadot-api/signer";
import { getPolkadotSigner } from "polkadot-api/signer";
import { MultiAddress, type subtensor } from "@polkadot-api/descriptors";
import type { KeyringPair } from "@moonwall/util";
import { Keyring } from "@polkadot/keyring";
import { waitForBlocks } from "./staking.js";
import { waitForFinalizedBlocks } from "./transactions.js";
import { generateKeyringPair } from "./account.ts";

export const groupSharingConfigAndUsage = () => Enum("ConfigAndUsage");
export const groupSharingConfigOnly = () => Enum("ConfigOnly");

type RpcCapableClient = {
    _request(method: string, params: unknown[]): Promise<unknown>;
};

export const rateLimitTargetGroup = (groupId: number) => Enum("Group", groupId);

export const rateLimitKindExact = (limit: bigint | number) =>
    Enum("Exact", typeof limit === "bigint" ? Number(limit) : limit);

const TX_TIMEOUT = 30_000;

async function waitForFinalizedBlockAdvance(api: TypedApi<typeof subtensor>, count = 1): Promise<void> {
    await waitForFinalizedBlocks(api, count);
}

async function waitForSudoTransactionWithRetry(
    api: TypedApi<typeof subtensor>,
    tx: any,
    signer: KeyringPair,
    label: string,
    maxRetries = 1
): Promise<void> {
    let retries = 0;

    while (retries < maxRetries) {
        try {
            await waitForSudoTransactionCompletion(api, tx, signer, label);
            return;
        } catch (error) {
            retries += 1;
            if (retries >= maxRetries) {
                throw new Error(`[${label}] failed after ${maxRetries} retries`);
            }
            await waitForBlocks(api, 1);
        }
    }
}

async function waitForSudoTransactionCompletion(
    api: TypedApi<typeof subtensor>,
    tx: any,
    keypair: KeyringPair,
    label: string
): Promise<void> {
    const signer = getPolkadotSigner(keypair.publicKey, "Sr25519", keypair.sign);
    const account = await api.query.System.Account.getValue(keypair.address, { at: "best" });

    return new Promise((resolve, reject) => {
        let timeoutId: ReturnType<typeof setTimeout>;
        const subscription = tx
            .signSubmitAndWatch(signer, {
                at: "best",
                nonce: account.nonce,
            })
            .subscribe({
                next: async (event: any) => {
                    if (event.type === "txBestBlocksState" && event.found) {
                        subscription.unsubscribe();

                        if (!event.ok) {
                            reject(new Error(`[${label}] dispatch error: ${JSON.stringify(event.dispatchError)}`));
                            return;
                        }

                        try {
                            const events = await api.query.System.Events.getValue({ at: event.block.hash });
                            const sudoEvent = events.find(
                                (record: any) =>
                                    record.phase?.type === "ApplyExtrinsic" &&
                                    record.phase.value === event.block.index &&
                                    record.event?.type === "Sudo" &&
                                    record.event?.value?.type === "Sudid"
                            ) as any;

                            const sudoResult = sudoEvent?.event?.value?.value?.sudo_result;
                            if (sudoResult?.success === false) {
                                reject(new Error(`[${label}] sudo error: ${JSON.stringify(sudoResult.value)}`));
                                return;
                            }

                            clearTimeout(timeoutId);
                            resolve();
                        } catch (error) {
                            clearTimeout(timeoutId);
                            reject(error instanceof Error ? error : new Error(String(error)));
                        }

                        return;
                    }

                    if (event.type === "txBestBlocksState" && event.isValid === false) {
                        subscription.unsubscribe();
                        clearTimeout(timeoutId);
                        reject(new Error(`[${label}] transaction rejected before inclusion`));
                    }
                },
                error: (error: unknown) => {
                    subscription.unsubscribe();
                    clearTimeout(timeoutId);
                    reject(error instanceof Error ? error : new Error(String(error)));
                },
            });

        timeoutId = setTimeout(() => {
            subscription.unsubscribe();
            reject(new Error(`[${label}] timeout`));
        }, TX_TIMEOUT);
    });
}

export async function waitForRateLimitTransactionWithRetry(
    api: TypedApi<typeof subtensor>,
    tx: any,
    signer: KeyringPair,
    label: string,
    maxRetries = 1
): Promise<void> {
    let retries = 0;

    while (retries < maxRetries) {
        try {
            await waitForRateLimitTransactionCompletion(tx, signer);
            return;
        } catch (error) {
            retries += 1;
            if (retries >= maxRetries) {
                throw new Error(`[${label}] failed after ${maxRetries} retries`);
            }
            await waitForBlocks(api, 1);
        }
    }
}

async function waitForRateLimitTransactionCompletion(
    tx: any,
    keypair: KeyringPair,
    timeout: number | null = TX_TIMEOUT
): Promise<{ txHash: string; blockHash: string }> {
    const signer = getPolkadotSigner(keypair.publicKey, "Sr25519", keypair.sign);

    const signSubmitAndWatchInner = (): Promise<{ txHash: string; blockHash: string }> =>
        new Promise((resolve, reject) => {
            const subscription = tx.signSubmitAndWatch(signer).subscribe({
                next(event: any) {
                    if (event.type === "txBestBlocksState" && event.found) {
                        subscription.unsubscribe();
                        if (event.dispatchError) {
                            reject(new Error(`ExtrinsicFailed: ${JSON.stringify(event.dispatchError)}`));
                        } else {
                            resolve({
                                txHash: event.txHash,
                                blockHash: event.block.hash,
                            });
                        }
                    } else if (event.type === "txBestBlocksState" && event.isValid === false) {
                        subscription.unsubscribe();
                        reject(new Error("Transaction rejected before inclusion"));
                    }
                },
                error(err: unknown) {
                    reject(err instanceof Error ? err : new Error(String(err)));
                },
            });
        });

    if (timeout === null) {
        return signSubmitAndWatchInner();
    }

    return new Promise((resolve, reject) => {
        const timer = setTimeout(() => reject(new Error("Transaction timed out")), timeout);
        signSubmitAndWatchInner()
            .then((result) => {
                clearTimeout(timer);
                resolve(result);
            })
            .catch((error) => {
                clearTimeout(timer);
                reject(error instanceof Error ? error : new Error(String(error)));
            });
    });
}

export async function forceSetBalancesForRateLimit(
    api: TypedApi<typeof subtensor>,
    ss58Addresses: string[],
    amount: bigint = 10000000000000000000n
): Promise<void> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const calls = ss58Addresses.map((ss58Address) =>
        api.tx.Balances.force_set_balance({
            who: MultiAddress.Id(ss58Address),
            new_free: amount,
        }).decodedCall
    );
    const batch = api.tx.Utility.force_batch({ calls });
    const tx = api.tx.Sudo.sudo({ call: batch.decodedCall });
    await waitForSudoTransactionWithRetry(api, tx, alice, "force_set_balance");
}

export async function addNewSubnetworkForRateLimit(
    api: TypedApi<typeof subtensor>,
    hotkey: KeyringPair,
    coldkey: KeyringPair
): Promise<number> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const totalNetworks = await api.query.SubtensorModule.TotalNetworks.getValue();

    const target = Enum("Group", 3);
    const limits = (await api.query.RateLimiting.Limits.getValue(target as never)) as any;
    const rateLimit =
        limits?.type === "Global" && limits.value?.type === "Exact" ? BigInt(limits.value.value) : BigInt(0);

    if (rateLimit !== BigInt(0)) {
        const internalCall = api.tx.RateLimiting.set_rate_limit({
            target: target as never,
            scope: undefined,
            limit: Enum("Exact", 0) as never,
        });
        const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall });
        await waitForSudoTransactionWithRetry(api, tx, alice, "set_network_rate_limit");
    }

    const registerNetworkTx = api.tx.SubtensorModule.register_network({
        hotkey: hotkey.address,
    });
    await waitForRateLimitTransactionWithRetry(api, registerNetworkTx, coldkey, "register_network");

    return totalNetworks;
}

export async function startCallForRateLimit(
    api: TypedApi<typeof subtensor>,
    netuid: number,
    coldkey: KeyringPair
): Promise<void> {
    const existingFirstEmission = await api.query.SubtensorModule.FirstEmissionBlockNumber.getValue(netuid);
    if (existingFirstEmission !== undefined) {
        return;
    }

    const registerBlock = Number(await api.query.SubtensorModule.NetworkRegisteredAt.getValue(netuid));
    let currentBlock = await api.query.System.Number.getValue();
    const duration = Number(await api.constants.SubtensorModule.InitialStartCallDelay);

    while (currentBlock - registerBlock <= duration) {
        await new Promise((resolve) => setTimeout(resolve, 2000));
        currentBlock = await api.query.System.Number.getValue();
    }

    await new Promise((resolve) => setTimeout(resolve, 2000));

    const tx = api.tx.SubtensorModule.start_call({ netuid });
    try {
        await waitForRateLimitTransactionWithRetry(api, tx, coldkey, "start_call");
    } catch (error) {
        if (error instanceof Error && error.message.includes("FirstEmissionBlockNumberAlreadySet")) {
            return;
        }
        throw error;
    }

    await new Promise((resolve) => setTimeout(resolve, 1000));
}

async function waitForGroupAtFinalized(api: TypedApi<typeof subtensor>, groupId: number, timeoutMs = 30_000): Promise<void> {
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

export async function createRateLimitGroup(api: TypedApi<typeof subtensor>, name: string, sharing: any): Promise<number> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
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
    label: string
): Promise<void> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const internalCalls = calls.map((call) =>
        api.tx.RateLimiting.register_call({
            call: call.decodedCall as never,
            group: groupId,
        }).decodedCall
    );
    const batch = api.tx.Utility.batch_all({ calls: internalCalls });
    const tx = api.tx.Sudo.sudo({ call: batch.decodedCall });
    await waitForSudoTransactionWithRetry(api, tx, alice, label);
    await waitForFinalizedBlockAdvance(api);
}

export async function setGlobalGroupRateLimit(
    api: TypedApi<typeof subtensor>,
    groupId: number,
    limit: bigint | number
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

    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
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
    limit: bigint | number
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

    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const internalCall = api.tx.RateLimiting.set_rate_limit({
        target: target as never,
        scope,
        limit: rateLimitKindExact(limit) as never,
    });
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall });
    await waitForSudoTransactionWithRetry(api, tx, alice, `set_scoped_group_rate_limit_${groupId}`);
    await waitForFinalizedBlockAdvance(api);
}

export async function getCallRateLimit(client: RpcCapableClient, pallet: string, extrinsic: string): Promise<any> {
    const encoder = new TextEncoder();
    return client._request("rateLimiting_getRateLimit", [
        Array.from(encoder.encode(pallet)),
        Array.from(encoder.encode(extrinsic)),
        null,
    ]);
}

export function getGroupedResponseGroupId(response: any): number | undefined {
    if (response && typeof response === "object" && "group_id" in response) {
        return Number((response as any).group_id);
    }
    if (response?.type === "grouped" || response?.type === "Grouped") {
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
        if (
            response.type === "grouped" ||
            response.type === "standalone" ||
            response.type === "Grouped" ||
            response.type === "Standalone"
        ) {
            return response.value?.limit;
        }
        if ("Grouped" in response) {
            return (response as any).Grouped.limit;
        }
        if ("Standalone" in response) {
            return (response as any).Standalone.limit;
        }
        const [key, value] = Object.entries(response)[0] ?? [];
        if (typeof key === "string" && (key.toLowerCase() === "grouped" || key.toLowerCase() === "standalone")) {
            return (value as any)?.limit;
        }
    }
    return undefined;
}

export function isScopedConfig(config: any): boolean {
    return Boolean(config && ((typeof config === "object" && "Scoped" in config) || config.type === "Scoped"));
}

export function isGlobalConfig(config: any): boolean {
    return Boolean(config && ((typeof config === "object" && "Global" in config) || config.type === "Global"));
}

export async function rootRegister(
    api: TypedApi<typeof subtensor>,
    coldkey: KeyringPair,
    hotkeyAddress: string
): Promise<void> {
    const tx = api.tx.SubtensorModule.root_register({ hotkey: hotkeyAddress });
    await waitForRateLimitTransactionWithRetry(api, tx, coldkey, "root_register");
}

export type RootHotkeyContext = {
    coldkey: KeyringPair;
    hotkey: KeyringPair;
    coldkeyAddress: string;
    hotkeyAddress: string;
};

export async function createRootHotkeyContext(api: TypedApi<typeof subtensor>): Promise<RootHotkeyContext> {
    const coldkey = generateKeyringPair("sr25519");
    const hotkey = generateKeyringPair("sr25519");
    const coldkeyAddress = coldkey.address;
    const hotkeyAddress = hotkey.address;

    await forceSetBalancesForRateLimit(api, [coldkeyAddress, hotkeyAddress]);
    await rootRegister(api, coldkey, hotkeyAddress);

    return { coldkey, hotkey, coldkeyAddress, hotkeyAddress };
}

export type OwnedSubnetContext = {
    coldkey: KeyringPair;
    hotkey: KeyringPair;
    coldkeyAddress: string;
    hotkeyAddress: string;
    netuid: number;
};

export async function createOwnedSubnetContext(api: TypedApi<typeof subtensor>): Promise<OwnedSubnetContext> {
    const coldkey = generateKeyringPair("sr25519");
    const hotkey = generateKeyringPair("sr25519");
    const coldkeyAddress = coldkey.address;
    const hotkeyAddress = hotkey.address;

    await forceSetBalancesForRateLimit(api, [coldkeyAddress, hotkeyAddress]);

    const netuid = await addNewSubnetworkForRateLimit(api, hotkey, coldkey);
    await startCallForRateLimit(api, netuid, coldkey);

    return { coldkey, hotkey, coldkeyAddress, hotkeyAddress, netuid };
}

export async function expectTransactionFailure(
    tx: any,
    signer: PolkadotSigner,
    label: string,
    timeoutMs = 20_000
): Promise<unknown> {
    return new Promise((resolve, reject) => {
        let settled = false;
        let timeoutId: ReturnType<typeof setTimeout>;

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
                        finish(() => reject(new Error(`[${label}] succeeded unexpectedly with tx ${value.txHash}`)));
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

        timeoutId = setTimeout(() => {
            subscription.unsubscribe();
            finish(() => reject(new Error(`[${label}] timed out waiting for failure`)));
        }, timeoutMs);
    });
}
