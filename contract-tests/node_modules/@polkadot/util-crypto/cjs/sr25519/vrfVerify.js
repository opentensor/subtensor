"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sr25519VrfVerify = sr25519VrfVerify;
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
const EMPTY_U8A = new Uint8Array();
/**
 * @name sr25519VrfVerify
 * @description Verify with sr25519 vrf verification
 */
function sr25519VrfVerify(message, signOutput, publicKey, context = EMPTY_U8A, extra = EMPTY_U8A) {
    const publicKeyU8a = (0, util_1.u8aToU8a)(publicKey);
    const proofU8a = (0, util_1.u8aToU8a)(signOutput);
    if (publicKeyU8a.length !== 32) {
        throw new Error('Invalid publicKey, expected 32-bytes');
    }
    else if (proofU8a.length !== 96) {
        throw new Error('Invalid vrfSign output, expected 96 bytes');
    }
    return (0, wasm_crypto_1.vrfVerify)(publicKeyU8a, (0, util_1.u8aToU8a)(context), (0, util_1.u8aToU8a)(message), (0, util_1.u8aToU8a)(extra), proofU8a);
}
