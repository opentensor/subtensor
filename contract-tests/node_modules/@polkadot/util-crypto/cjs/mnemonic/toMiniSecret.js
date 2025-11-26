"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.mnemonicToMiniSecret = mnemonicToMiniSecret;
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
const index_js_1 = require("../pbkdf2/index.js");
const toEntropy_js_1 = require("./toEntropy.js");
const validate_js_1 = require("./validate.js");
function mnemonicToMiniSecret(mnemonic, password = '', wordlist, onlyJs) {
    if (!(0, validate_js_1.mnemonicValidate)(mnemonic, wordlist, onlyJs)) {
        throw new Error('Invalid bip39 mnemonic specified');
    }
    else if (!wordlist && !onlyJs && (0, wasm_crypto_1.isReady)()) {
        return (0, wasm_crypto_1.bip39ToMiniSecret)(mnemonic, password);
    }
    const entropy = (0, toEntropy_js_1.mnemonicToEntropy)(mnemonic, wordlist);
    const salt = (0, util_1.stringToU8a)(`mnemonic${password}`);
    // return the first 32 bytes as the seed
    return (0, index_js_1.pbkdf2Encode)(entropy, salt).password.slice(0, 32);
}
