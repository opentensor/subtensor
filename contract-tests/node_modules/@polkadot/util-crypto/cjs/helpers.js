"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createAsHex = createAsHex;
exports.createBitHasher = createBitHasher;
exports.createDualHasher = createDualHasher;
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
/** @internal */
function createAsHex(fn) {
    return (...args) => (0, util_1.u8aToHex)(fn(...args));
}
/** @internal */
function createBitHasher(bitLength, fn) {
    return (data, onlyJs) => fn(data, bitLength, onlyJs);
}
/** @internal */
function createDualHasher(wa, js) {
    return (value, bitLength = 256, onlyJs) => {
        const u8a = (0, util_1.u8aToU8a)(value);
        return !util_1.hasBigInt || (!onlyJs && (0, wasm_crypto_1.isReady)())
            ? wa[bitLength](u8a)
            : js[bitLength](u8a);
    };
}
