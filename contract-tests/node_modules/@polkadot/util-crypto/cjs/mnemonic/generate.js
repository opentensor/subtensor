"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.mnemonicGenerate = mnemonicGenerate;
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
const bip39_js_1 = require("./bip39.js");
/**
 * @name mnemonicGenerate
 * @summary Creates a valid mnemonic string using using [BIP39](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki).
 * @example
 * <BR>
 *
 * ```javascript
 * import { mnemonicGenerate } from '@polkadot/util-crypto';
 *
 * const mnemonic = mnemonicGenerate(); // => string
 * ```
 */
function mnemonicGenerate(numWords = 12, wordlist, onlyJs) {
    return !util_1.hasBigInt || (!wordlist && !onlyJs && (0, wasm_crypto_1.isReady)())
        ? (0, wasm_crypto_1.bip39Generate)(numWords)
        : (0, bip39_js_1.generateMnemonic)(numWords, wordlist);
}
