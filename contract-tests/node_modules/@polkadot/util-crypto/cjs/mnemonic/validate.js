"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.mnemonicValidate = mnemonicValidate;
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
const bip39_js_1 = require("./bip39.js");
/**
 * @name mnemonicValidate
 * @summary Validates a mnemonic input using [BIP39](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki).
 * @example
 * <BR>
 *
 * ```javascript
 * import { mnemonicGenerate, mnemonicValidate } from '@polkadot/util-crypto';
 *
 * const mnemonic = mnemonicGenerate(); // => string
 * const isValidMnemonic = mnemonicValidate(mnemonic); // => boolean
 * ```
 */
function mnemonicValidate(mnemonic, wordlist, onlyJs) {
    return !util_1.hasBigInt || (!wordlist && !onlyJs && (0, wasm_crypto_1.isReady)())
        ? (0, wasm_crypto_1.bip39Validate)(mnemonic)
        : (0, bip39_js_1.validateMnemonic)(mnemonic, wordlist);
}
