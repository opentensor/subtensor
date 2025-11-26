"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.bestNumberLag = bestNumberLag;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
/**
 * @name bestNumberLag
 * @returns A number of blocks
 * @description Calculates the lag between finalized head and best head
 * @examplew
 * ```javascript
 * api.derive.chain.bestNumberLag((lag) => {
 *   console.log(`finalized is ${lag} blocks behind head`);
 * });
 * ```
 */
function bestNumberLag(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => (0, rxjs_1.combineLatest)([
        api.derive.chain.bestNumber(),
        api.derive.chain.bestNumberFinalized()
    ]).pipe((0, rxjs_1.map)(([bestNumber, bestNumberFinalized]) => api.registry.createType('BlockNumber', bestNumber.sub(bestNumberFinalized)))));
}
