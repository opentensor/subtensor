"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.cryptoIsReady = void 0;
exports.cryptoWaitReady = cryptoWaitReady;
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
exports.cryptoIsReady = wasm_crypto_1.isReady;
function cryptoWaitReady() {
    return (0, wasm_crypto_1.waitReady)()
        .then(() => {
        if (!(0, wasm_crypto_1.isReady)()) {
            throw new Error('Unable to initialize @polkadot/util-crypto');
        }
        return true;
    })
        .catch(() => false);
}
