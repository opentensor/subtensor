"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sr25519VrfSign = sr25519VrfSign;
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
const EMPTY_U8A = new Uint8Array();
/**
 * @name sr25519VrfSign
 * @description Sign with sr25519 vrf signing (deterministic)
 */
function sr25519VrfSign(message, { secretKey }, context = EMPTY_U8A, extra = EMPTY_U8A) {
    if (secretKey?.length !== 64) {
        throw new Error('Invalid secretKey, expected 64-bytes');
    }
    return (0, wasm_crypto_1.vrfSign)(secretKey, (0, util_1.u8aToU8a)(context), (0, util_1.u8aToU8a)(message), (0, util_1.u8aToU8a)(extra));
}
