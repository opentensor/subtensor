import { secp256k1 } from '@noble/curves/secp256k1';
import { bnToU8a, hasBigInt, u8aConcat } from '@polkadot/util';
import { isReady, secp256k1Expand as wasm } from '@polkadot/wasm-crypto';
import { BN_BE_256_OPTS } from '../bn.js';
export function secp256k1Expand(publicKey, onlyJs) {
    if (![33, 65].includes(publicKey.length)) {
        throw new Error(`Invalid publicKey provided, received ${publicKey.length} bytes input`);
    }
    if (publicKey.length === 65) {
        return publicKey.subarray(1);
    }
    if (!hasBigInt || (!onlyJs && isReady())) {
        return wasm(publicKey).subarray(1);
    }
    const { px, py } = secp256k1.ProjectivePoint.fromHex(publicKey);
    return u8aConcat(bnToU8a(px, BN_BE_256_OPTS), bnToU8a(py, BN_BE_256_OPTS));
}
