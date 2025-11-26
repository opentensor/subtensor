"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sr25519DerivePublic = sr25519DerivePublic;
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
function sr25519DerivePublic(publicKey, chainCode) {
    const publicKeyU8a = (0, util_1.u8aToU8a)(publicKey);
    if (!(0, util_1.isU8a)(chainCode) || chainCode.length !== 32) {
        throw new Error('Invalid chainCode passed to derive');
    }
    else if (publicKeyU8a.length !== 32) {
        throw new Error(`Invalid publicKey, received ${publicKeyU8a.length} bytes, expected 32`);
    }
    return (0, wasm_crypto_1.sr25519DerivePublicSoft)(publicKeyU8a, chainCode);
}
