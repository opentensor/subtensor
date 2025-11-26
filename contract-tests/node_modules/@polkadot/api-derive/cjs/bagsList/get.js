"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports._getIds = _getIds;
exports.all = all;
exports.get = get;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
const util_js_1 = require("./util.js");
function orderBags(ids, bags) {
    const sorted = ids
        .map((id, index) => ({
        bag: bags[index].unwrapOr(null),
        id,
        key: id.toString()
    }))
        .sort((a, b) => b.id.cmp(a.id));
    const max = sorted.length - 1;
    return sorted.map((entry, index) => (0, util_1.objectSpread)(entry, {
        bagLower: index === max
            ? util_1.BN_ZERO
            : sorted[index + 1].id,
        bagUpper: entry.id,
        index
    }));
}
function _getIds(instanceId, api) {
    const query = (0, util_js_1.getQueryInterface)(api);
    return (0, index_js_1.memo)(instanceId, (_ids) => {
        const ids = _ids.map((id) => (0, util_1.bnToBn)(id));
        return ids.length
            ? query.listBags.multi(ids).pipe((0, rxjs_1.map)((bags) => orderBags(ids, bags)))
            : (0, rxjs_1.of)([]);
    });
}
function all(instanceId, api) {
    const query = (0, util_js_1.getQueryInterface)(api);
    return (0, index_js_1.memo)(instanceId, () => query.listBags.keys().pipe((0, rxjs_1.switchMap)((keys) => api.derive.bagsList._getIds(keys.map(({ args: [id] }) => id))), (0, rxjs_1.map)((list) => list.filter(({ bag }) => bag))));
}
/**
 * @name get
 * @param {(BN | number)} id The id of the bag to retrieve.
 * @description Retrieves a specific bag from the BagsList pallet by its id.
 */
function get(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (id) => api.derive.bagsList._getIds([(0, util_1.bnToBn)(id)]).pipe((0, rxjs_1.map)((bags) => bags[0])));
}
