import { log } from "./logger.js";
import type { KeyringPair } from "@moonwall/util";
import { sleep } from "@zombienet/utils";
import { waitForBlocks } from "./staking.ts";
import type { Transaction, TypedApi } from "polkadot-api";
import type { subtensor } from "@polkadot-api/descriptors";
import { getPolkadotSigner } from "polkadot-api/signer";

export const TX_TIMEOUT = 30_000;

export async function waitForTransactionWithRetry(
    api: TypedApi<typeof subtensor>,
    tx: Transaction<Record<string, unknown>, string, string, void>,
    signer: KeyringPair,
    label: string,
    maxRetries = 1
): Promise<void> {
    let retries = 0;

    while (retries < maxRetries) {
        try {
            await waitForTransactionCompletion(tx, signer);
            return;
        } catch (error) {
            log.tx(label, `attempt ${retries + 1} failed: ${error}`);
            retries += 1;
            if (retries >= maxRetries) {
                throw new Error(`[${label}] failed after ${maxRetries} retries`);
            }
            await waitForBlocks(api, 1);
        }
    }
}

export async function waitForTransactionCompletion(
    tx: Transaction<Record<string, unknown>, string, string, void>,
    keypair: KeyringPair,
    timeout: number | null = TX_TIMEOUT
): Promise<{ txHash: string; blockHash: string }> {
    const callerStack = new Error().stack;

    const signer = getPolkadotSigner(keypair.publicKey, "Sr25519", keypair.sign);

    const signSubmitAndWatchInner = (): Promise<{ txHash: string; blockHash: string }> => {
        return new Promise((resolve, reject) => {
            const subscription = tx.signSubmitAndWatch(signer).subscribe({
                next(event) {
                    if (event.type === "finalized") {
                        subscription.unsubscribe();

                        const failed = event.dispatchError;
                        if (failed) {
                            reject(new Error(`ExtrinsicFailed: ${JSON.stringify(failed)}`));
                        } else {
                            resolve({
                                txHash: event.txHash,
                                blockHash: event.block.hash,
                            });
                        }
                    }
                },
                error(err) {
                    console.error("callerStack", callerStack);
                    reject(err instanceof Error ? err : new Error(String(err)));
                },
            });
        });
    };

    if (timeout === null) {
        return signSubmitAndWatchInner();
    }

    return new Promise((resolve, reject) => {
        const timer = setTimeout(() => {
            console.log("Transaction timed out");
            console.error("callerStack", callerStack);
            reject(new Error("Transaction timed out"));
        }, timeout);

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

export async function waitForSudoTransactionWithRetry(
    api: TypedApi<typeof subtensor>,
    tx: Transaction<Record<string, unknown>, string, string, void>,
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
            log.tx(label, `error: ${error}`);
            retries += 1;
            if (retries >= maxRetries) {
                throw new Error(`[${label}] failed after ${maxRetries} retries`);
            }
            await waitForBlocks(api, 1);
        }
    }
}

const SECOND = 1000;

/** Polls the chain until `count` new finalized blocks have been produced. */
export async function waitForFinalizedBlocks(
    api: TypedApi<typeof subtensor>,
    count: number,
    pollInterval = 1 * SECOND,
    timeout = 120 * SECOND
): Promise<void> {
    const startBlock = await api.query.System.Number.getValue({ at: "finalized" });
    const target = startBlock + count;
    const deadline = Date.now() + timeout;

    while (Date.now() < deadline) {
        await sleep(pollInterval);

        const currentBlock = await api.query.System.Number.getValue({ at: "finalized" });

        if (currentBlock >= target) return;
    }

    throw new Error(`Timed out waiting for ${count} finalized blocks (from #${startBlock}, target #${target})`);
}

export async function waitForFinalizedBlockAdvance(
    api: TypedApi<typeof subtensor>,
    count = 1,
    pollInterval = 1 * SECOND,
    timeout = 120 * SECOND
): Promise<void> {
    await waitForFinalizedBlocks(api, count, pollInterval, timeout);
}

async function waitForSudoTransactionCompletion(
    api: TypedApi<typeof subtensor>,
    tx: Transaction<Record<string, unknown>, string, string, void>,
    keypair: KeyringPair,
    label: string
): Promise<void> {
    const signer = getPolkadotSigner(keypair.publicKey, "Sr25519", keypair.sign);
    const signerAddress = keypair.address;
    const account = await api.query.System.Account.getValue(signerAddress, { at: "best" });

    return new Promise((resolve, reject) => {
        let txHash = "";
        let timeoutId: ReturnType<typeof setTimeout>;
        const subscription = tx
            .signSubmitAndWatch(signer, {
                at: "best",
                nonce: account.nonce,
            })
            .subscribe({
                next: async (value) => {
                    txHash = value.txHash;

                    if (value.type === "txBestBlocksState" && value.found) {
                        subscription.unsubscribe();

                        if (!value.ok) {
                            const errorStr = JSON.stringify(value.dispatchError, null, 2);
                            log.tx(label, `dispatch error: ${errorStr}`);
                            reject(new Error(`[${label}] dispatch error: ${errorStr}`));
                            return;
                        }

                        try {
                            const events = await api.query.System.Events.getValue({ at: value.block.hash });
                            const sudoEvent = events.find(
                                (eventRecord: any) =>
                                    eventRecord.phase?.type === "ApplyExtrinsic" &&
                                    eventRecord.phase.value === value.block.index &&
                                    eventRecord.event?.type === "Sudo" &&
                                    eventRecord.event?.value?.type === "Sudid"
                            ) as any;

                            const sudoResult = sudoEvent?.event?.value?.value?.sudo_result;
                            if (sudoResult?.success === false) {
                                const errorStr = JSON.stringify(sudoResult.value, null, 2);
                                log.tx(label, `sudo error: ${errorStr}`);
                                reject(new Error(`[${label}] sudo error: ${errorStr}`));
                                return;
                            }

                            log.tx(label, `included: ${value.txHash}`);
                            clearTimeout(timeoutId);
                            resolve();
                        } catch (error) {
                            clearTimeout(timeoutId);
                            reject(error instanceof Error ? error : new Error(String(error)));
                        }

                        return;
                    }

                    if (value.type === "txBestBlocksState" && value.isValid === false) {
                        subscription.unsubscribe();
                        clearTimeout(timeoutId);
                        reject(new Error(`[${label}] transaction rejected before inclusion`));
                    }
                },
                error: (error) => {
                    subscription.unsubscribe();
                    clearTimeout(timeoutId);
                    reject(error instanceof Error ? error : new Error(String(error)));
                },
            });

        timeoutId = setTimeout(() => {
            subscription.unsubscribe();
            log.tx(label, `timeout for tx: ${txHash}`);
            reject(new Error(`[${label}] timeout`));
        }, TX_TIMEOUT);
    });
}
