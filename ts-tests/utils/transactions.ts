import { log } from "./logger.js";
import type { KeyringPair } from "@moonwall/util";
import { sleep } from "@zombienet/utils";
import { waitForBlocks } from "./staking.ts";
import type { Transaction, TypedApi } from "polkadot-api";
import type { subtensor } from "@polkadot-api/descriptors";
import { getPolkadotSigner } from "polkadot-api/signer";

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
    timeout: number | null = 3 * 60 * 1000
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

export type TransactionResult = {
    success: boolean;
    events: any[];
    txHash?: string;
    blockHash?: string;
    errorMessage?: string;
};

/**
 * Send a transaction and return a result object instead of throwing on ExtrinsicFailed.
 * Use this for tests that expect failure.
 */
export async function sendTransaction(
    api: ApiPromise,
    tx: SubmittableExtrinsic,
    signer: KeyringPair,
    timeout: number = 3 * 60 * 1000,
): Promise<TransactionResult> {
    return new Promise((resolve, reject) => {
        const timer = setTimeout(() => {
            reject(new Error("Transaction timed out"));
        }, timeout);

        let unsub: () => void;

        tx.signAndSend(signer, (result) => {
            const { status, txHash } = result;
            if (status.isFinalized) {
                clearTimeout(timer);
                unsub?.();

                const failed = result.events.find(({ event }) => api.events.system.ExtrinsicFailed.is(event));

                if (failed) {
                    const { dispatchError } = failed.event.data as any;
                    let errorMessage = dispatchError.toString();
                    if (dispatchError.isModule) {
                        const decoded = api.registry.findMetaError(dispatchError.asModule);
                        errorMessage = `${decoded.section}.${decoded.name}: ${decoded.docs.join(" ")}`;
                    }
                    resolve({
                        success: false,
                        events: result.events.map((e) => e.event),
                        txHash: txHash.toHex(),
                        blockHash: status.asFinalized.toHex(),
                        errorMessage,
                    });
                } else {
                    resolve({
                        success: true,
                        events: result.events.map((e) => e.event),
                        txHash: txHash.toHex(),
                        blockHash: status.asFinalized.toHex(),
                    });
                }
            }
        })
            .then((u) => {
                unsub = u;
            })
            .catch((error) => {
                clearTimeout(timer);
                const message = error instanceof Error ? error.message : String(error?.toHuman?.() ?? error);
                resolve({ success: false, events: [], errorMessage: message });
            });
    });
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
