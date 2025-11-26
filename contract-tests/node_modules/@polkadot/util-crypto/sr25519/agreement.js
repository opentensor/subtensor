import { u8aToU8a } from '@polkadot/util';
import { sr25519Agree } from '@polkadot/wasm-crypto';
/**
 * @name sr25519Agreement
 * @description Key agreement between other's public key and self secret key
 */
export function sr25519Agreement(secretKey, publicKey) {
    const secretKeyU8a = u8aToU8a(secretKey);
    const publicKeyU8a = u8aToU8a(publicKey);
    if (publicKeyU8a.length !== 32) {
        throw new Error(`Invalid publicKey, received ${publicKeyU8a.length} bytes, expected 32`);
    }
    else if (secretKeyU8a.length !== 64) {
        throw new Error(`Invalid secretKey, received ${secretKeyU8a.length} bytes, expected 64`);
    }
    return sr25519Agree(publicKeyU8a, secretKeyU8a);
}
