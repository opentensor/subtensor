"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sqrtElectorate = sqrtElectorate;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
/**
 * @name sqrtElectorate
 * @description Computes the square root of the total token issuance in the network.
 * @example
 * ```javascript
 * let sqrtElectorate = await api.derive.democracy.sqrtElectorate();
 * console.log("Square root of token issuance:", sqrtElectorate);
 * ```
 */
function sqrtElectorate(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => api.query.balances.totalIssuance().pipe((0, rxjs_1.map)(util_1.bnSqrt)));
}
