"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.mnemonicToSeedSync = mnemonicToSeedSync;
exports.mnemonicToEntropy = mnemonicToEntropy;
exports.entropyToMnemonic = entropyToMnemonic;
exports.generateMnemonic = generateMnemonic;
exports.validateMnemonic = validateMnemonic;
const tslib_1 = require("tslib");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../pbkdf2/index.js");
const index_js_2 = require("../random/index.js");
const index_js_3 = require("../sha/index.js");
const en_js_1 = tslib_1.__importDefault(require("./wordlists/en.js"));
const INVALID_MNEMONIC = 'Invalid mnemonic';
const INVALID_ENTROPY = 'Invalid entropy';
const INVALID_CHECKSUM = 'Invalid mnemonic checksum';
/** @internal */
function normalize(str) {
    return (str || '').normalize('NFKD');
}
/** @internal */
function binaryToByte(bin) {
    return parseInt(bin, 2);
}
/** @internal */
function bytesToBinary(bytes) {
    return bytes.map((x) => x.toString(2).padStart(8, '0')).join('');
}
/** @internal */
function deriveChecksumBits(entropyBuffer) {
    return bytesToBinary(Array.from((0, index_js_3.sha256AsU8a)(entropyBuffer))).slice(0, (entropyBuffer.length * 8) / 32);
}
function mnemonicToSeedSync(mnemonic, password) {
    return (0, index_js_1.pbkdf2Encode)((0, util_1.stringToU8a)(normalize(mnemonic)), (0, util_1.stringToU8a)(`mnemonic${normalize(password)}`)).password;
}
function mnemonicToEntropy(mnemonic, wordlist = en_js_1.default) {
    const words = normalize(mnemonic).split(' ');
    if (words.length % 3 !== 0) {
        throw new Error(INVALID_MNEMONIC);
    }
    // convert word indices to 11 bit binary strings
    const bits = words
        .map((word) => {
        const index = wordlist.indexOf(word);
        if (index === -1) {
            throw new Error(INVALID_MNEMONIC);
        }
        return index.toString(2).padStart(11, '0');
    })
        .join('');
    // split the binary string into ENT/CS
    const dividerIndex = Math.floor(bits.length / 33) * 32;
    const entropyBits = bits.slice(0, dividerIndex);
    const checksumBits = bits.slice(dividerIndex);
    // calculate the checksum and compare
    const matched = entropyBits.match(/(.{1,8})/g);
    const entropyBytes = matched?.map(binaryToByte);
    if (!entropyBytes || (entropyBytes.length % 4 !== 0) || (entropyBytes.length < 16) || (entropyBytes.length > 32)) {
        throw new Error(INVALID_ENTROPY);
    }
    const entropy = (0, util_1.u8aToU8a)(entropyBytes);
    if (deriveChecksumBits(entropy) !== checksumBits) {
        throw new Error(INVALID_CHECKSUM);
    }
    return entropy;
}
function entropyToMnemonic(entropy, wordlist = en_js_1.default) {
    // 128 <= ENT <= 256
    if ((entropy.length % 4 !== 0) || (entropy.length < 16) || (entropy.length > 32)) {
        throw new Error(INVALID_ENTROPY);
    }
    const matched = `${bytesToBinary(Array.from(entropy))}${deriveChecksumBits(entropy)}`.match(/(.{1,11})/g);
    const mapped = matched?.map((b) => wordlist[binaryToByte(b)]);
    if (!mapped || (mapped.length < 12)) {
        throw new Error('Unable to map entropy to mnemonic');
    }
    return mapped.join(' ');
}
function generateMnemonic(numWords, wordlist) {
    return entropyToMnemonic((0, index_js_2.randomAsU8a)((numWords / 3) * 4), wordlist);
}
function validateMnemonic(mnemonic, wordlist) {
    try {
        mnemonicToEntropy(mnemonic, wordlist);
    }
    catch {
        return false;
    }
    return true;
}
