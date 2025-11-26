"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.initWasm = initWasm;
const both_1 = require("@polkadot/wasm-crypto-init/both");
const init_js_1 = require("./init.js");
/**
 * @name initWasm
 * @description
 * For historic purposes and for tighter control on init, specifically performing
 * a WASM initialization with ASM and an ASM.js fallback
 *
 * Generally should not be used unless you want explicit control over which
 * interfaces are initialized.
 */
async function initWasm() {
    await (0, init_js_1.initBridge)(both_1.createWasm);
}
initWasm().catch(() => {
    // cannot happen, initWasm doesn't throw
});
