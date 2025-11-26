"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.bridge = void 0;
exports.initBridge = initBridge;
const wasm_bridge_1 = require("@polkadot/wasm-bridge");
const wasm_crypto_init_1 = require("@polkadot/wasm-crypto-init");
/**
 * @name bridge
 * @description
 * The JS <-> WASM bridge that is in operation. For the specific package
 * it is a global, i.e. all operations happens on this specific bridge
 */
exports.bridge = new wasm_bridge_1.Bridge(wasm_crypto_init_1.createWasm);
/**
 * @name initBridge
 * @description
 * Creates a new bridge interface with the (optional) initialization function
 */
async function initBridge(createWasm) {
    return exports.bridge.init(createWasm);
}
