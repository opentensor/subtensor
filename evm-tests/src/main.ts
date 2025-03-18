
import { createClient, TypedApi, Transaction, PolkadotSigner, Binary } from 'polkadot-api';
import { devnet, MultiAddress } from '@polkadot-api/descriptors';
import { getDevnetApi, getAlice } from './substrate';
import { convertPublicKeyToSs58 } from './address-utils'

export async function getNonceChangePromise() {
    const api = await getDevnetApi()
    const ss58Address = convertPublicKeyToSs58(getAlice().publicKey)
    // api.query.System.Account.getValue()
    const initValue = await api.query.System.Account.getValue(ss58Address);
    console.log("init nonce is ", initValue.nonce)
    return new Promise<void>((resolve, reject) => {
        const subscription = api.query.System.Account.watchValue(ss58Address).subscribe({
            next(value) {
                console.log("in main, new nonce is ", value.nonce)
                // if (value.nonce > initValue.nonce) {
                // subscription.unsubscribe();
                // Resolve the promise when the transaction is finalized
                // console.log("will resolve nonce promise")
                // resolve();
                // }
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

    })
}

getNonceChangePromise()


