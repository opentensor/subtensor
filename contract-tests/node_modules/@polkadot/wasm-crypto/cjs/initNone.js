"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.initWasm = initWasm;
const none_1 = require("@polkadot/wasm-crypto-init/none");
const init_js_1 = require("./init.js");
/**
 * @name initWasm
 * @description
 * For historic purposes and for tighter control on init, specifically performing
 * a WASM initialization with no interface whatsoever (no WASM, no ASM.js)
 *
 * Generally should not be used unless you want explicit control over which
 * interfaces are initialized.
 */
async function initWasm() {
    await (0, init_js_1.initBridge)(none_1.createWasm);
}
initWasm().catch(() => {
    // cannot happen, initWasm doesn't throw
});
