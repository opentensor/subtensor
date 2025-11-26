"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.currentPoints = currentPoints;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
/**
 * @name currentPoints
 * @description Retrieve the staking overview, including elected and points earned.
 * @example
 * ```javascript
 * const currentPoints = await api.derive.staking.currentPoints();
 * console.log(currentPoints.toHuman());
 * ```
 */
function currentPoints(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => api.derive.session.indexes().pipe((0, rxjs_1.switchMap)(({ activeEra }) => api.query.staking.erasRewardPoints(activeEra))));
}
