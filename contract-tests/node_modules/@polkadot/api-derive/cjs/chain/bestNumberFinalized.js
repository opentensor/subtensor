"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.bestNumberFinalized = void 0;
const util_js_1 = require("./util.js");
/**
 * @name bestNumberFinalized
 * @returns A BlockNumber
 * @description Get the latest finalized block number.
 * @example
 * ```javascript
 * api.derive.chain.bestNumberFinalized((blockNumber) => {
 *   console.log(`the current finalized block is #${blockNumber}`);
 * });
 * ```
 */
exports.bestNumberFinalized = (0, util_js_1.createBlockNumberDerive)((api) => api.rpc.chain.subscribeFinalizedHeads());
