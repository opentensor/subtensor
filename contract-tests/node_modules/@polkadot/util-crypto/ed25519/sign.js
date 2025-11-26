import { ed25519 } from '@noble/curves/ed25519';
import { hasBigInt, u8aToU8a } from '@polkadot/util';
import { ed25519Sign as wasmSign, isReady } from '@polkadot/wasm-crypto';
/**
 * @name ed25519Sign
 * @summary Signs a message using the supplied secretKey
 * @description
 * Returns message signature of `message`, using the `secretKey`.
 * @example
 * <BR>
 *
 * ```javascript
 * import { ed25519Sign } from '@polkadot/util-crypto';
 *
 * ed25519Sign([...], [...]); // => [...]
 * ```
 */
export function ed25519Sign(message, { publicKey, secretKey }, onlyJs) {
    if (!secretKey) {
        throw new Error('Expected a valid secretKey');
    }
    else if (!publicKey) {
        throw new Error('Expected a valid publicKey');
    }
    const messageU8a = u8aToU8a(message);
    const privateU8a = secretKey.subarray(0, 32);
    return !hasBigInt || (!onlyJs && isReady())
        ? wasmSign(publicKey, privateU8a, messageU8a)
        : ed25519.sign(messageU8a, privateU8a);
}
