"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.bestNumber = void 0;
const util_js_1 = require("./util.js");
/**
 * @name bestNumber
 * @descrive Retrieves the latest block number.
 * @example
 * ```javascript
 * api.derive.chain.bestNumber((blockNumber) => {
 *   console.log(`the current best block is #${blockNumber}`);
 * });
 * ```
 */
exports.bestNumber = (0, util_js_1.createBlockNumberDerive)((api) => api.rpc.chain.subscribeNewHeads());
