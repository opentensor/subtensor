import { secp256k1 } from '@noble/curves/secp256k1';
import { bnToU8a, hasBigInt, u8aConcat } from '@polkadot/util';
import { isReady, secp256k1Sign as wasm } from '@polkadot/wasm-crypto';
import { BN_BE_256_OPTS } from '../bn.js';
import { hasher } from './hasher.js';
/**
 * @name secp256k1Sign
 * @description Returns message signature of `message`, using the supplied pair
 */
export function secp256k1Sign(message, { secretKey }, hashType = 'blake2', onlyJs) {
    if (secretKey?.length !== 32) {
        throw new Error('Expected valid secp256k1 secretKey, 32-bytes');
    }
    const data = hasher(hashType, message, onlyJs);
    if (!hasBigInt || (!onlyJs && isReady())) {
        return wasm(data, secretKey);
    }
    const signature = secp256k1.sign(data, secretKey, { lowS: true });
    return u8aConcat(bnToU8a(signature.r, BN_BE_256_OPTS), bnToU8a(signature.s, BN_BE_256_OPTS), new Uint8Array([signature.recovery || 0]));
}
