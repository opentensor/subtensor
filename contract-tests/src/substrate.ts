import { devnet, MultiAddress } from '@polkadot-api/descriptors';
import { TypedApi, Transaction, PolkadotSigner, Binary } from 'polkadot-api';
import { sr25519CreateDerive } from "@polkadot-labs/hdkd"
import { DEV_PHRASE, entropyToMiniSecret, mnemonicToEntropy, KeyPair } from "@polkadot-labs/hdkd-helpers"
import { getPolkadotSigner } from "polkadot-api/signer"
import { randomBytes } from 'crypto';
import { Keyring } from '@polkadot/keyring';
import { SS58_PREFIX, TX_TIMEOUT } from "./config";
import { getClient } from "./setup"

let api: TypedApi<typeof devnet> | undefined = undefined

// define url string as type to extend in the future
// export type ClientUrlType = 'ws://localhost:9944' | 'wss://test.finney.opentensor.ai:443' | 'wss://dev.chain.opentensor.ai:443' | 'wss://archive.chain.opentensor.ai';
export type ClientUrlType = 'ws://localhost:9944'

export async function getDevnetApi() {
    if (api === undefined) {
        let client = await getClient()

        api = client.getTypedApi(devnet)
    }
    return api
}

export function getKeypairFromPath(path: string) {
    const entropy = mnemonicToEntropy(DEV_PHRASE)
    const miniSecret = entropyToMiniSecret(entropy)
    const derive = sr25519CreateDerive(miniSecret)
    const hdkdKeyPair = derive(path)

    return hdkdKeyPair
}

export const getAlice = () => getKeypairFromPath("//Alice")
export const getBob = () => getKeypairFromPath("//Bob")
export const getCharlie = () => getKeypairFromPath("//Charlie")
export const getDave = () => getKeypairFromPath("//Dave")

export function getSignerFromPath(path: string) {
    const keypair = getKeypairFromPath(path)
    const polkadotSigner = getPolkadotSigner(
        keypair.publicKey,
        "Sr25519",
        keypair.sign,
    )

    return polkadotSigner
}

export const getAliceSigner = () => getSignerFromPath("//Alice")
export const getBobSigner = () => getSignerFromPath("//Bob")
export const getCharlieSigner = () => getSignerFromPath("//Charlie")
export const getDaveSigner = () => getSignerFromPath("//Dave")

export function getRandomSubstrateSigner() {
    const keypair = getRandomSubstrateKeypair();
    return getSignerFromKeypair(keypair)
}

export function getSignerFromKeypair(keypair: KeyPair) {
    const polkadotSigner = getPolkadotSigner(
        keypair.publicKey,
        "Sr25519",
        keypair.sign,
    )
    return polkadotSigner
}

export function getRandomSubstrateKeypair() {
    const seed = randomBytes(32);
    const miniSecret = entropyToMiniSecret(seed)
    const derive = sr25519CreateDerive(miniSecret)
    const hdkdKeyPair = derive("")

    return hdkdKeyPair
}

export async function getBalance(api: TypedApi<typeof devnet>, ss58Address: string) {
    const value = await api.query.System.Account.getValue(ss58Address)
    return value.data.free
}

export async function getNonce(api: TypedApi<typeof devnet>, ss58Address: string): Promise<number> {
    const value = await api.query.System.Account.getValue(ss58Address);
    return value.nonce
}

export async function getNonceChangePromise(api: TypedApi<typeof devnet>, ss58Address: string) {
    // api.query.System.Account.getValue()
    const initValue = await api.query.System.Account.getValue(ss58Address);
    return new Promise<void>((resolve, reject) => {
        const subscription = api.query.System.Account.watchValue(ss58Address).subscribe({
            next(value) {
                if (value.nonce > initValue.nonce) {
                    subscription.unsubscribe();
                    // Resolve the promise when the transaction is finalized
                    resolve();
                }
            },

            error(err: Error) {
                console.error("Transaction failed:", err);
                subscription.unsubscribe();
                // Reject the promise in case of an error
                reject(err);
            },
            complete() {
                console.log("Subscription complete");
            }
        })

        setTimeout(() => {
            subscription.unsubscribe();
            console.log('unsubscribed!');
            resolve()
        }, TX_TIMEOUT);

    })
}

export function convertPublicKeyToMultiAddress(publicKey: Uint8Array, ss58Format: number = SS58_PREFIX): MultiAddress {
    // Create a keyring instance
    const keyring = new Keyring({ type: 'sr25519', ss58Format });

    // Add the public key to the keyring
    const address = keyring.encodeAddress(publicKey);

    return MultiAddress.Id(address);
}

export async function waitForTransactionWithRetry(
    api: TypedApi<typeof devnet>,
    tx: Transaction<{}, string, string, void>,
    signer: PolkadotSigner,
) {
    let success = false;
    let retries = 0;

    // set max retries times
    while (!success && retries < 5) {
        await waitForTransactionCompletion(api, tx, signer)
            .then(() => { success = true })
            .catch((error) => {
                console.log(`transaction error ${error}`);
            });
        await new Promise((resolve) => setTimeout(resolve, 1000));
        retries += 1;
    }

    if (!success) {
        console.log("Transaction failed after 5 retries");
    }
}

export async function waitForTransactionCompletion(api: TypedApi<typeof devnet>, tx: Transaction<{}, string, string, void>, signer: PolkadotSigner,) {
    const transactionPromise = await getTransactionWatchPromise(tx, signer)
    return transactionPromise

    // If we can't always get the finalized event, then add nonce subscribe as other evidence for tx is finalized.
    // Don't need it based on current testing.
    // const ss58Address = convertPublicKeyToSs58(signer.publicKey)
    // const noncePromise = await getNonceChangePromise(api, ss58Address)

    // return new Promise<void>((resolve, reject) => {
    //     Promise.race([transactionPromise, noncePromise])
    //         .then(resolve)
    //         .catch(reject);
    // })
}


export async function getTransactionWatchPromise(tx: Transaction<{}, string, string, void>, signer: PolkadotSigner,) {
    return new Promise<void>((resolve, reject) => {
        // store the txHash, then use it in timeout. easier to know which tx is not finalized in time
        let txHash = ""
        const subscription = tx.signSubmitAndWatch(signer).subscribe({
            next(value) {
                txHash = value.txHash

                // TODO investigate why finalized not for each extrinsic
                if (value.type === "finalized") {
                    console.log("Transaction is finalized in block:", value.txHash);
                    subscription.unsubscribe();
                    clearTimeout(timeoutId);
                    if (!value.ok) {
                        console.log("Transaction threw an error:", value.dispatchError)
                    }
                    // Resolve the promise when the transaction is finalized
                    resolve();
                }
            },
            error(err) {
                console.error("Transaction failed:", err);
                subscription.unsubscribe();
                clearTimeout(timeoutId);
                // Reject the promise in case of an error
                reject(err);

            },
            complete() {
                console.log("Subscription complete");
            }
        });

        const timeoutId = setTimeout(() => {
            subscription.unsubscribe();
            console.log('unsubscribed because of timeout for tx {}', txHash);
            reject()
        }, TX_TIMEOUT);
    });
}

// second solution to wait for transaction finalization. pass the raw data to avoid the complex transaction type definition
export async function waitForTransactionCompletion2(api: TypedApi<typeof devnet>, raw: Binary, signer: PolkadotSigner,) {
    const tx = await api.txFromCallData(raw);
    return new Promise<void>((resolve, reject) => {
        const subscription = tx.signSubmitAndWatch(signer).subscribe({
            next(value) {
                console.log("Event:", value);

                if (value.type === "txBestBlocksState") {
                    console.log("Transaction is finalized in block:", value.txHash);
                    subscription.unsubscribe();
                    // Resolve the promise when the transaction is finalized
                    resolve();

                }
            },
            error(err: Error) {
                console.error("Transaction failed:", err);
                subscription.unsubscribe();
                // Reject the promise in case of an error
                reject(err);

            },
            complete() {
                console.log("Subscription complete");
            }
        });
    });
}

export async function waitForNonceChange(api: TypedApi<typeof devnet>, ss58Address: string) {
    const initNonce = await getNonce(api, ss58Address)
    while (true) {
        const currentNonce = await getNonce(api, ss58Address)
        if (currentNonce > initNonce) {
            break
        }

        await new Promise(resolve => setTimeout(resolve, 200));
    }
}

export function waitForFinalizedBlock(api: TypedApi<typeof devnet>, end: number) {
    return new Promise<void>((resolve) => {
        const subscription = api.query.System.Number.watchValue("finalized").subscribe((current) => {
            if (current > end) {
                subscription.unsubscribe();
                resolve();
            }
        })
    })
}