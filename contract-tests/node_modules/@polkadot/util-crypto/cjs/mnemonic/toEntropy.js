"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.mnemonicToEntropy = mnemonicToEntropy;
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
const bip39_js_1 = require("./bip39.js");
function mnemonicToEntropy(mnemonic, wordlist, onlyJs) {
    return !util_1.hasBigInt || (!wordlist && !onlyJs && (0, wasm_crypto_1.isReady)())
        ? (0, wasm_crypto_1.bip39ToEntropy)(mnemonic)
        : (0, bip39_js_1.mnemonicToEntropy)(mnemonic, wordlist);
}
