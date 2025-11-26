import { u8aToU8a } from '@polkadot/util';
import { vrfVerify } from '@polkadot/wasm-crypto';
const EMPTY_U8A = new Uint8Array();
/**
 * @name sr25519VrfVerify
 * @description Verify with sr25519 vrf verification
 */
export function sr25519VrfVerify(message, signOutput, publicKey, context = EMPTY_U8A, extra = EMPTY_U8A) {
    const publicKeyU8a = u8aToU8a(publicKey);
    const proofU8a = u8aToU8a(signOutput);
    if (publicKeyU8a.length !== 32) {
        throw new Error('Invalid publicKey, expected 32-bytes');
    }
    else if (proofU8a.length !== 96) {
        throw new Error('Invalid vrfSign output, expected 96 bytes');
    }
    return vrfVerify(publicKeyU8a, u8aToU8a(context), u8aToU8a(message), u8aToU8a(extra), proofU8a);
}
