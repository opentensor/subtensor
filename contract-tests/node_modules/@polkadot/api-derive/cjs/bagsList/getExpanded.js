"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.expand = expand;
exports.getExpanded = getExpanded;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
/**
 * @name expand
 * @description Expands a given bag by retrieving all its nodes (accounts contained within the bag).
 * @param {Bag} bag The bag to be expanded.
 */
function expand(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (bag) => api.derive.bagsList.listNodes(bag.bag).pipe((0, rxjs_1.map)((nodes) => (0, util_1.objectSpread)({ nodes }, bag))));
}
/**
 * @name getExpanded
 * @description Retrieves and expands a specific bag from the BagsList pallet.
 * @param {BN | number} id The id of the bag to expand.
 */
function getExpanded(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (id) => api.derive.bagsList.get(id).pipe((0, rxjs_1.switchMap)((bag) => api.derive.bagsList.expand(bag))));
}
