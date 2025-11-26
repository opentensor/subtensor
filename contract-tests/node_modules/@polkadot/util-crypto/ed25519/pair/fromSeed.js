import { ed25519 } from '@noble/curves/ed25519';
import { hasBigInt, u8aConcatStrict } from '@polkadot/util';
import { ed25519KeypairFromSeed, isReady } from '@polkadot/wasm-crypto';
/**
 * @name ed25519PairFromSeed
 * @summary Creates a new public/secret keypair from a seed.
 * @description
 * Returns a object containing a `publicKey` & `secretKey` generated from the supplied seed.
 * @example
 * <BR>
 *
 * ```javascript
 * import { ed25519PairFromSeed } from '@polkadot/util-crypto';
 *
 * ed25519PairFromSeed(...); // => { secretKey: [...], publicKey: [...] }
 * ```
 */
export function ed25519PairFromSeed(seed, onlyJs) {
    if (!hasBigInt || (!onlyJs && isReady())) {
        const full = ed25519KeypairFromSeed(seed);
        return {
            publicKey: full.slice(32),
            secretKey: full.slice(0, 64)
        };
    }
    const publicKey = ed25519.getPublicKey(seed);
    return {
        publicKey,
        secretKey: u8aConcatStrict([seed, publicKey])
    };
}
