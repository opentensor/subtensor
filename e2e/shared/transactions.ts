import { subtensor } from "@polkadot-api/descriptors";
import { TypedApi, Transaction, PolkadotSigner } from "polkadot-api";
import { ss58Address } from "@polkadot-labs/hdkd-helpers";
import { log } from "./logger.js";

export const TX_TIMEOUT = 30_000;

export async function waitForTransactionWithRetry(
  api: TypedApi<typeof subtensor>,
  tx: Transaction<{}, string, string, void>,
  signer: PolkadotSigner,
  label: string,
  maxRetries = 1,
): Promise<void> {
  let success = false;
  let retries = 0;

  while (!success && retries < maxRetries) {
    await waitForTransactionCompletion(api, tx, signer, label)
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

export async function waitForTransactionFinalizedWithRetry(
  api: TypedApi<typeof subtensor>,
  tx: Transaction<{}, string, string, void>,
  signer: PolkadotSigner,
  label: string,
  maxRetries = 1,
): Promise<void> {
  let success = false;
  let retries = 0;

  while (!success && retries < maxRetries) {
    await waitForTransactionFinalizedCompletion(api, tx, signer, label)
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

export async function waitForSudoTransactionWithRetry(
  api: TypedApi<typeof subtensor>,
  tx: Transaction<{}, string, string, void>,
  signer: PolkadotSigner,
  label: string,
  maxRetries = 1,
): Promise<void> {
  let success = false;
  let retries = 0;

  while (!success && retries < maxRetries) {
    await waitForSudoTransactionCompletion(api, tx, signer, label)
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

export async function waitForFinalizedBlockAdvance(
  api: TypedApi<typeof subtensor>,
  count = 1,
  pollMs = 1_000,
  timeoutMs = TX_TIMEOUT,
): Promise<void> {
  const start = Number(await api.query.System.Number.getValue({ at: "finalized" }));
  const target = start + count;
  const deadline = Date.now() + timeoutMs;

  while (Date.now() < deadline) {
    await new Promise((resolve) => setTimeout(resolve, pollMs));
    const current = Number(await api.query.System.Number.getValue({ at: "finalized" }));
    if (current >= target) {
      return;
    }
  }

  throw new Error(`Timed out waiting for finalized block #${target}`);
}

async function waitForTransactionCompletion(
  api: TypedApi<typeof subtensor>,
  tx: Transaction<{}, string, string, void>,
  signer: PolkadotSigner,
  label: string,
): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    const signerAddress = ss58Address(signer.publicKey);
    const txOptionsPromise = api.query.System.Account.getValue(signerAddress, { at: "best" }).then(
      (account) => ({
        at: "best" as const,
        nonce: account.nonce,
      }),
    );

    let txHash = "";
    let subscription:
      | {
          unsubscribe(): void;
        }
      | undefined;

    txOptionsPromise
      .then((txOptions) => {
        subscription = tx.signSubmitAndWatch(signer, txOptions).subscribe({
          next(value) {
            txHash = value.txHash;
            if (value.type === "txBestBlocksState" && value.found) {
              log.tx(label, `included: ${value.txHash}`);
              subscription?.unsubscribe();
              clearTimeout(timeoutId);
              if (!value.ok) {
                const errorStr = JSON.stringify(value.dispatchError, null, 2);
                log.tx(label, `dispatch error: ${errorStr}`);
                reject(new Error(`[${label}] dispatch error: ${errorStr}`));
              } else {
                resolve();
              }
            } else if (value.type === "txBestBlocksState" && value.isValid === false) {
              subscription?.unsubscribe();
              clearTimeout(timeoutId);
              reject(new Error(`[${label}] transaction rejected before inclusion`));
            }
          },
          error(err) {
            log.error(label, `failed: ${err}`);
            subscription?.unsubscribe();
            clearTimeout(timeoutId);
            reject(err);
          },
        });
      })
      .catch((error) => {
        clearTimeout(timeoutId);
        reject(error);
      });

    const timeoutId = setTimeout(() => {
      subscription?.unsubscribe();
      log.tx(label, `timeout for tx: ${txHash}`);
      reject(new Error(`[${label}] timeout`));
    }, TX_TIMEOUT);
  });
}

async function waitForTransactionFinalizedCompletion(
  api: TypedApi<typeof subtensor>,
  tx: Transaction<{}, string, string, void>,
  signer: PolkadotSigner,
  label: string,
): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    const signerAddress = ss58Address(signer.publicKey);
    const txOptionsPromise = api.query.System.Account.getValue(signerAddress, { at: "best" }).then(
      (account) => ({
        at: "best" as const,
        nonce: account.nonce,
      }),
    );

    let txHash = "";
    let subscription:
      | {
          unsubscribe(): void;
        }
      | undefined;

    txOptionsPromise
      .then((txOptions) => {
        subscription = tx.signSubmitAndWatch(signer, txOptions).subscribe({
          next(value) {
            txHash = value.txHash;
            if (value.type === "finalized") {
              log.tx(label, `finalized: ${value.txHash}`);
              subscription?.unsubscribe();
              clearTimeout(timeoutId);
              if (!value.ok) {
                const errorStr = JSON.stringify(value.dispatchError, null, 2);
                log.tx(label, `dispatch error: ${errorStr}`);
                reject(new Error(`[${label}] dispatch error: ${errorStr}`));
              } else {
                resolve();
              }
            } else if (value.type === "txBestBlocksState" && value.isValid === false) {
              subscription?.unsubscribe();
              clearTimeout(timeoutId);
              reject(new Error(`[${label}] transaction rejected before inclusion`));
            }
          },
          error(err) {
            log.error(label, `failed: ${err}`);
            subscription?.unsubscribe();
            clearTimeout(timeoutId);
            reject(err);
          },
        });
      })
      .catch((error) => {
        clearTimeout(timeoutId);
        reject(error);
      });

    const timeoutId = setTimeout(() => {
      subscription?.unsubscribe();
      log.tx(label, `timeout for tx: ${txHash}`);
      reject(new Error(`[${label}] timeout`));
    }, TX_TIMEOUT);
  });
}

async function waitForSudoTransactionCompletion(
  api: TypedApi<typeof subtensor>,
  tx: Transaction<{}, string, string, void>,
  signer: PolkadotSigner,
  label: string,
): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    const signerAddress = ss58Address(signer.publicKey);
    const txOptionsPromise = api.query.System.Account.getValue(signerAddress, { at: "best" }).then(
      (account) => ({
        at: "best" as const,
        nonce: account.nonce,
      }),
    );

    let txHash = "";
    let subscription:
      | {
          unsubscribe(): void;
        }
      | undefined;

    txOptionsPromise
      .then((txOptions) => {
        subscription = tx.signSubmitAndWatch(signer, txOptions).subscribe({
          async next(value) {
            txHash = value.txHash;
            if (value.type === "txBestBlocksState" && value.found) {
              log.tx(label, `included: ${value.txHash}`);
              subscription?.unsubscribe();
              clearTimeout(timeoutId);

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
                    eventRecord.event?.value?.type === "Sudid",
                ) as any;

                const sudoResult = sudoEvent?.event?.value?.value?.sudo_result;
                if (sudoResult?.success === false) {
                  const errorStr = JSON.stringify(sudoResult.value, null, 2);
                  log.tx(label, `sudo error: ${errorStr}`);
                  reject(new Error(`[${label}] sudo error: ${errorStr}`));
                  return;
                }

                resolve();
              } catch (error) {
                reject(error);
              }
            } else if (value.type === "txBestBlocksState" && value.isValid === false) {
              subscription?.unsubscribe();
              clearTimeout(timeoutId);
              reject(new Error(`[${label}] transaction rejected before inclusion`));
            }
          },
          error(err) {
            log.error(label, `failed: ${err}`);
            subscription?.unsubscribe();
            clearTimeout(timeoutId);
            reject(err);
          },
        });
      })
      .catch((error) => {
        clearTimeout(timeoutId);
        reject(error);
      });

    const timeoutId = setTimeout(() => {
      subscription?.unsubscribe();
      log.tx(label, `timeout for tx: ${txHash}`);
      reject(new Error(`[${label}] timeout`));
    }, TX_TIMEOUT);
  });
}
