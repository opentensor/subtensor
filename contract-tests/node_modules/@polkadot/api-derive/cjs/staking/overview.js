"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.overview = overview;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
/**
 * @name overview
 * @description Retrieve the staking overview, including elected validators and points earned.
 * @example
 * ```javascript
 * const {
 *   activeEra,
 *   activeEraStart,
 *   currentEra,
 *   currentIndex,
 *   nextElected,
 *   validatorCount,
 *   validators,
 * } = await api.derive.staking.overview();
 * ```
 */
function overview(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => (0, rxjs_1.combineLatest)([
        api.derive.session.indexes(),
        api.derive.staking.validators()
    ]).pipe((0, rxjs_1.map)(([indexes, { nextElected, validators }]) => (0, util_1.objectSpread)({}, indexes, {
        nextElected,
        validators
    }))));
}
