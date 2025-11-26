"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.mnemonicToLegacySeed = mnemonicToLegacySeed;
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
const bip39_js_1 = require("./bip39.js");
const validate_js_1 = require("./validate.js");
/**
 * @name mnemonicToLegacySeed
 * @summary Creates a valid Ethereum/Bitcoin-compatible seed from a mnemonic input
 * @example
 * <BR>
 *
 * ```javascript
 * import { mnemonicGenerate, mnemonicToLegacySeed, mnemonicValidate } from '@polkadot/util-crypto';
 *
 * const mnemonic = mnemonicGenerate(); // => string
 * const isValidMnemonic = mnemonicValidate(mnemonic); // => boolean
 *
 * if (isValidMnemonic) {
 *   console.log(`Seed generated from mnemonic: ${mnemonicToLegacySeed(mnemonic)}`); => u8a
 * }
 * ```
 */
function mnemonicToLegacySeed(mnemonic, password = '', onlyJs, byteLength = 32) {
    if (!(0, validate_js_1.mnemonicValidate)(mnemonic)) {
        throw new Error('Invalid bip39 mnemonic specified');
    }
    else if (![32, 64].includes(byteLength)) {
        throw new Error(`Invalid seed length ${byteLength}, expected 32 or 64`);
    }
    return byteLength === 32
        ? !util_1.hasBigInt || (!onlyJs && (0, wasm_crypto_1.isReady)())
            ? (0, wasm_crypto_1.bip39ToSeed)(mnemonic, password)
            : (0, bip39_js_1.mnemonicToSeedSync)(mnemonic, password).subarray(0, 32)
        : (0, bip39_js_1.mnemonicToSeedSync)(mnemonic, password);
}
