"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.contributions = contributions;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
const util_js_1 = require("./util.js");
const PAGE_SIZE_K = 1000; // limit aligned with the 1k on the node (trie lookups are heavy)
function _getUpdates(api, paraId) {
    let added = [];
    let removed = [];
    return api.query.system.events().pipe((0, rxjs_1.switchMap)((events) => {
        const changes = (0, util_js_1.extractContributed)(paraId, events);
        if (changes.added.length || changes.removed.length) {
            added = added.concat(...changes.added);
            removed = removed.concat(...changes.removed);
            return (0, rxjs_1.of)({ added, addedDelta: changes.added, blockHash: events.createdAtHash?.toHex() || '-', removed, removedDelta: changes.removed });
        }
        return rxjs_1.EMPTY;
    }), (0, rxjs_1.startWith)({ added, addedDelta: [], blockHash: '-', removed, removedDelta: [] }));
}
function _eventTriggerAll(api, paraId) {
    return api.query.system.events().pipe((0, rxjs_1.switchMap)((events) => {
        const items = events.filter(({ event: { data: [eventParaId], method, section } }) => section === 'crowdloan' &&
            ['AllRefunded', 'Dissolved', 'PartiallyRefunded'].includes(method) &&
            eventParaId.eq(paraId));
        return items.length
            ? (0, rxjs_1.of)(events.createdAtHash?.toHex() || '-')
            : rxjs_1.EMPTY;
    }), (0, rxjs_1.startWith)('-'));
}
function _getKeysPaged(api, childKey) {
    const subject = new rxjs_1.BehaviorSubject(undefined);
    return subject.pipe((0, rxjs_1.switchMap)((startKey) => api.rpc.childstate.getKeysPaged(childKey, '0x', PAGE_SIZE_K, startKey)), (0, rxjs_1.tap)((keys) => {
        (0, util_1.nextTick)(() => {
            keys.length === PAGE_SIZE_K
                ? subject.next(keys[PAGE_SIZE_K - 1].toHex())
                : subject.complete();
        });
    }), (0, rxjs_1.toArray)(), // toArray since we want to startSubject to be completed
    (0, rxjs_1.map)((keyArr) => (0, util_1.arrayFlatten)(keyArr)));
}
function _getAll(api, paraId, childKey) {
    return _eventTriggerAll(api, paraId).pipe((0, rxjs_1.switchMap)(() => (0, util_1.isFunction)(api.rpc.childstate.getKeysPaged)
        ? _getKeysPaged(api, childKey)
        : api.rpc.childstate.getKeys(childKey, '0x')), (0, rxjs_1.map)((keys) => keys.map((k) => k.toHex())));
}
function _contributions(api, paraId, childKey) {
    return (0, rxjs_1.combineLatest)([
        _getAll(api, paraId, childKey),
        _getUpdates(api, paraId)
    ]).pipe((0, rxjs_1.map)(([keys, { added, blockHash, removed }]) => {
        const contributorsMap = {};
        keys.forEach((k) => {
            contributorsMap[k] = true;
        });
        added.forEach((k) => {
            contributorsMap[k] = true;
        });
        removed.forEach((k) => {
            delete contributorsMap[k];
        });
        return {
            blockHash,
            contributorsHex: Object.keys(contributorsMap)
        };
    }));
}
/**
 * @name contributions
 * @description Retrieves all contributions for a given parachain crowdloan.
 * @param {string | number | BN} paraId The parachain ID for which contributions are being queried.
 * @example
 * ```javascript
 * const contributions = await api.derive.crowdloan.contributions(3369);
 * console.log("Contributions:", contributions);
 * ```
 */
function contributions(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (paraId) => api.derive.crowdloan.childKey(paraId).pipe((0, rxjs_1.switchMap)((childKey) => childKey
        ? _contributions(api, paraId, childKey)
        : (0, rxjs_1.of)({ blockHash: '-', contributorsHex: [] }))));
}
