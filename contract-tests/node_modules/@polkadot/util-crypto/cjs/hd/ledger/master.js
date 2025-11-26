"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ledgerMaster = ledgerMaster;
const util_1 = require("@polkadot/util");
const index_js_1 = require("../../hmac/index.js");
const bip39_js_1 = require("../../mnemonic/bip39.js");
const ED25519_CRYPTO = 'ed25519 seed';
function ledgerMaster(mnemonic, password) {
    const seed = (0, bip39_js_1.mnemonicToSeedSync)(mnemonic, password);
    const chainCode = (0, index_js_1.hmacShaAsU8a)(ED25519_CRYPTO, new Uint8Array([1, ...seed]), 256);
    let priv;
    while (!priv || (priv[31] & 0b0010_0000)) {
        priv = (0, index_js_1.hmacShaAsU8a)(ED25519_CRYPTO, priv || seed, 512);
    }
    priv[0] &= 0b1111_1000;
    priv[31] &= 0b0111_1111;
    priv[31] |= 0b0100_0000;
    return (0, util_1.u8aConcat)(priv, chainCode);
}
