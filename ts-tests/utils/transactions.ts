import { log } from "./logger.js";
import type { KeyringPair } from "@moonwall/util";
import type { SubmittableExtrinsic } from "@polkadot/api/promise/types";
import type { AddressOrPair } from "@polkadot/api-base/types/submittable";
import type { ApiPromise } from "@polkadot/api";
import { sleep } from "@zombienet/utils";
import { waitForBlocks } from "./staking.ts";

export async function waitForTransactionWithRetry(
    api: ApiPromise,
    tx: SubmittableExtrinsic,
    signer: KeyringPair,
    label: string,
    maxRetries = 1
): Promise<void> {
    let retries = 0;

    while (retries < maxRetries) {
        try {
            await waitForTransactionCompletion(api, tx, signer);
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
    api: ApiPromise,
    tx: SubmittableExtrinsic,
    account: AddressOrPair,
    timeout: number | null = 3 * 60 * 1000
) {
    const callerStack = new Error().stack;

    // Inner function that doesn't handle timeout
    const signAndSendAndIncludeInner = (tx: SubmittableExtrinsic, account: AddressOrPair) => {
        return new Promise((resolve, reject) => {
            let unsub: () => void;

            tx.signAndSend(account, (result) => {
                const { status, txHash } = result;
                // Resolve once the transaction is finalized
                if (status.isFinalized) {
                    // Uncomment if you need to debug transaction events
                    // console.debug(
                    //     "tx events:",
                    //     result.events.map((event) => JSON.stringify(event.toHuman()))
                    // );

                    const failed = result.events.find(({ event }) => api.events.system.ExtrinsicFailed.is(event));

                    unsub?.();
                    if (failed) {
                        const { dispatchError } = failed.event.data as any;
                        let errorMessage = dispatchError.toString();

                        if (dispatchError.isModule) {
                            const decoded = api.registry.findMetaError(dispatchError.asModule);
                            errorMessage = `${decoded.section}.${decoded.name}: ${decoded.docs.join(" ")}`;
                        }
                        reject(new Error(`ExtrinsicFailed: ${errorMessage}`));
                    } else {
                        resolve({ txHash, blockHash: status.asFinalized, status: result });
                    }
                }
            })
                .then((u) => {
                    unsub = u;
                })
                .catch((error) => {
                    console.error("callerStack", callerStack);
                    reject(error.toHuman());
                });
        });
    };

    // If no timeout is specified, directly call the no-timeout version
    if (timeout === null) {
        return signAndSendAndIncludeInner(tx, account);
    }

    // Otherwise, create our own promise that sets/rejects on timeout
    return new Promise((resolve, reject) => {
        const timer = setTimeout(() => {
            console.log("Transaction timed out");
            console.log(tx.toJSON());
            console.error("callerStack", callerStack);
            reject(new Error("Transaction timed out"));
        }, timeout);

        signAndSendAndIncludeInner(tx, account)
            .then((result) => {
                clearTimeout(timer);
                resolve(result);
            })
            .catch((error) => {
                clearTimeout(timer);
                // error может быть Error, string, или polkadot-объектом с .toHuman()
                if (error instanceof Error) {
                    reject(error);
                } else if (typeof error?.toHuman === "function") {
                    reject(new Error(JSON.stringify(error.toHuman())));
                } else {
                    reject(new Error(String(error)));
                }
            });
    });
}

const SECOND = 1000;

/** Polls the chain until `count` new finalized blocks have been produced. */
export async function waitForFinalizedBlocks(
    api: ApiPromise,
    count: number,
    pollInterval = 1 * SECOND,
    timeout = 120 * SECOND
): Promise<void> {
    const block = await api.rpc.chain.getBlock(await api.rpc.chain.getFinalizedHead());
    const start = block.block.header.number.toNumber();

    const target = start + count;
    const deadline = Date.now() + timeout;

    while (Date.now() < deadline) {
        await sleep(pollInterval);

        const currentBlock = await api.rpc.chain.getBlock(await api.rpc.chain.getFinalizedHead());
        const currentBlockNumber = currentBlock.block.header.number.toNumber();

        if (currentBlockNumber >= target) return;
    }

    throw new Error(`Timed out waiting for ${count} finalized blocks (from #${start}, target #${target})`);
}
