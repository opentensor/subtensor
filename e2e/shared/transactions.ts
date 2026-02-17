import { devnet } from "@polkadot-api/descriptors";
import { TypedApi, Transaction, PolkadotSigner } from "polkadot-api";
import { log } from "./logger.js";

export const TX_TIMEOUT = 5000;

export async function waitForTransactionWithRetry(
  api: TypedApi<typeof devnet>,
  tx: Transaction<{}, string, string, void>,
  signer: PolkadotSigner,
  label: string,
  maxRetries = 5
): Promise<void> {
  let success = false;
  let retries = 0;

  while (!success && retries < maxRetries) {
    await waitForTransactionCompletion(tx, signer, label)
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

async function waitForTransactionCompletion(
  tx: Transaction<{}, string, string, void>,
  signer: PolkadotSigner,
  label: string
): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    let txHash = "";
    const subscription = tx.signSubmitAndWatch(signer).subscribe({
      next(value) {
        txHash = value.txHash;
        if (value.type === "finalized") {
          log.tx(label, `finalized: ${value.txHash}`);
          subscription.unsubscribe();
          clearTimeout(timeoutId);
          if (!value.ok) {
            log.tx(label, `dispatch error: ${value.dispatchError}`);
          }
          resolve();
        }
      },
      error(err) {
        log.error(label, `failed: ${err}`);
        subscription.unsubscribe();
        clearTimeout(timeoutId);
        reject(err);
      },
    });

    const timeoutId = setTimeout(() => {
      subscription.unsubscribe();
      log.tx(label, `timeout for tx: ${txHash}`);
      reject(new Error(`[${label}] timeout`));
    }, TX_TIMEOUT);
  });
}
