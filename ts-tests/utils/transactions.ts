import { log } from "./logger.js";
import type { KeyringPair } from "@moonwall/util";
import type { SubmittableExtrinsic } from "@polkadot/api/promise/types";
import type { AddressOrPair } from "@polkadot/api-base/types/submittable";
import type { ApiPromise } from "@polkadot/api";
import { sleep } from "@zombienet/utils";

export async function waitForTransactionWithRetry(
    tx: SubmittableExtrinsic<"promise">,
    signer: KeyringPair,
    label: string,
    maxRetries = 1
): Promise<void> {
    let success = false;
    let retries = 0;

    while (!success && retries < maxRetries) {
        await waitForTransactionCompletion(tx, signer)
            .then(() => {
                success = true;
            })
            .catch((error) => {
                log.tx(label, `error: ${error}`);
            });
        await new Promise((resolve) => setTimeout(resolve, 1000));
        retries += 1;
    }

    if (!success) {
        throw new Error(`[${label}] failed after ${maxRetries} retries`);
    }
}

export async function waitForTransactionCompletion(
    tx: SubmittableExtrinsic<"promise">,
    account: AddressOrPair,
    timeout: number | null = 3 * 60 * 1000
) {
    const callerStack = new Error().stack;

    // Inner function that doesn't handle timeout
    const signAndSendAndIncludeInner = (tx: SubmittableExtrinsic<"promise">, account: AddressOrPair) => {
        return new Promise((resolve, reject) => {
            tx.signAndSend(account, (result) => {
                const { status, txHash } = result;

                // Resolve once the transaction is finalized
                if (status.isFinalized) {
                    // console.debug(
                    //     "tx events:",
                    //     result.events.map((event) => JSON.stringify(event.toHuman()))
                    // );
                    resolve({
                        txHash,
                        blockHash: status.asFinalized,
                        status: result,
                    });
                }
            }).catch((error) => {
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
                reject(error.toHuman());
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
