"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.initWasm = initWasm;
const asm_1 = require("@polkadot/wasm-crypto-init/asm");
const init_js_1 = require("./init.js");
/**
 * @name initWasm
 * @description
 * For historic purposes and for tighter control on init, specifically performing
 * a WASM initialization with only ASM.js
 *
 * Generally should not be used unless you want explicit control over which
 * interfaces are initialized.
 */
async function initWasm() {
    await (0, init_js_1.initBridge)(asm_1.createWasm);
}
initWasm().catch(() => {
    // cannot happen, initWasm doesn't throw
});
