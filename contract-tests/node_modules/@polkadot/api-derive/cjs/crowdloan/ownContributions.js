"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ownContributions = ownContributions;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
const util_js_1 = require("./util.js");
function _getValues(api, childKey, keys) {
    // We actually would love to use multi-keys https://github.com/paritytech/substrate/issues/9203
    return (0, rxjs_1.combineLatest)(keys.map((k) => api.rpc.childstate.getStorage(childKey, k))).pipe((0, rxjs_1.map)((values) => values
        .map((v) => api.registry.createType('Option<StorageData>', v))
        .map((o) => o.isSome
        ? api.registry.createType('Balance', o.unwrap())
        : api.registry.createType('Balance'))
        .reduce((all, b, index) => (0, util_1.objectSpread)(all, { [keys[index]]: b }), {})));
}
function _watchOwnChanges(api, paraId, childkey, keys) {
    return api.query.system.events().pipe((0, rxjs_1.switchMap)((events) => {
        const changes = (0, util_js_1.extractContributed)(paraId, events);
        const filtered = keys.filter((k) => changes.added.includes(k) ||
            changes.removed.includes(k));
        return filtered.length
            ? _getValues(api, childkey, filtered)
            : rxjs_1.EMPTY;
    }), (0, rxjs_1.startWith)({}));
}
function _contributions(api, paraId, childKey, keys) {
    return (0, rxjs_1.combineLatest)([
        _getValues(api, childKey, keys),
        _watchOwnChanges(api, paraId, childKey, keys)
    ]).pipe((0, rxjs_1.map)(([all, latest]) => (0, util_1.objectSpread)({}, all, latest)));
}
/**
 * @name ownContributions
 * @description Retrieves the contribution amounts made by specific accounts (`keys`) to a given parachain crowdloan (`paraId`).
 * @param {string | number | BN} paraId The parachain ID for which contributions are being queried.
 * @param {string[]} keys An array of account addresses whose contributions are to be fetched.
 * @example
 * ```javascript
 * const contributions = await api.derive.crowdloan.ownContributions(2000, ['5Ff...PqV', '5Gg...XyZ']);
 * console.log("Own Contributions:", contributions);
 * ```
 */
function ownContributions(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (paraId, keys) => api.derive.crowdloan.childKey(paraId).pipe((0, rxjs_1.switchMap)((childKey) => childKey && keys.length
        ? _contributions(api, paraId, childKey, keys)
        : (0, rxjs_1.of)({}))));
}
