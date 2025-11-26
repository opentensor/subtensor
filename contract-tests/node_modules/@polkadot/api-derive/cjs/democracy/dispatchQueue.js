"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.dispatchQueue = dispatchQueue;
const rxjs_1 = require("rxjs");
const types_1 = require("@polkadot/types");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
const util_js_1 = require("./util.js");
const DEMOCRACY_ID = (0, util_1.stringToHex)('democrac');
function isMaybeHashedOrBounded(call) {
    // check for enum
    return call instanceof types_1.Enum;
}
function isBounded(call) {
    // check for type
    return call.isInline || call.isLegacy || call.isLookup;
}
function queryQueue(api) {
    return api.query.democracy['dispatchQueue']().pipe((0, rxjs_1.switchMap)((dispatches) => (0, rxjs_1.combineLatest)([
        (0, rxjs_1.of)(dispatches),
        api.derive.democracy.preimages(dispatches.map(([, hash]) => hash))
    ])), (0, rxjs_1.map)(([dispatches, images]) => dispatches.map(([at, imageHash, index], dispatchIndex) => ({
        at,
        image: images[dispatchIndex],
        imageHash: (0, util_js_1.getImageHashBounded)(imageHash),
        index
    }))));
}
function schedulerEntries(api) {
    // We don't get entries, but rather we get the keys (triggered via finished referendums) and
    // the subscribe to those keys - this means we pickup when the schedulers actually executes
    // at a block, the entry for that block will become empty
    return api.derive.democracy.referendumsFinished().pipe((0, rxjs_1.switchMap)(() => api.query.scheduler.agenda.keys()), (0, rxjs_1.switchMap)((keys) => {
        const blockNumbers = keys.map(({ args: [blockNumber] }) => blockNumber);
        return blockNumbers.length
            ? (0, rxjs_1.combineLatest)([
                (0, rxjs_1.of)(blockNumbers),
                // this should simply be api.query.scheduler.agenda.multi,
                // however we have had cases on Darwinia where the indices have moved around after an
                // upgrade, which results in invalid on-chain data
                api.query.scheduler.agenda.multi(blockNumbers).pipe((0, rxjs_1.catchError)(() => (0, rxjs_1.of)(blockNumbers.map(() => []))))
            ])
            : (0, rxjs_1.of)([[], []]);
    }));
}
function queryScheduler(api) {
    return schedulerEntries(api).pipe((0, rxjs_1.switchMap)(([blockNumbers, agendas]) => {
        const result = [];
        blockNumbers.forEach((at, index) => {
            (agendas[index] || []).filter((o) => o.isSome).forEach((o) => {
                const scheduled = o.unwrap();
                if (scheduled.maybeId.isSome) {
                    const id = scheduled.maybeId.unwrap().toHex();
                    if (id.startsWith(DEMOCRACY_ID)) {
                        const imageHash = isMaybeHashedOrBounded(scheduled.call)
                            ? isBounded(scheduled.call)
                                ? (0, util_js_1.getImageHashBounded)(scheduled.call)
                                : scheduled.call.isHash
                                    ? scheduled.call.asHash.toHex()
                                    : scheduled.call.asValue.args[0].toHex()
                            : scheduled.call.args[0].toHex();
                        result.push({ at, imageHash, index: api.registry.createType('(u64, ReferendumIndex)', id)[1] });
                    }
                }
            });
        });
        return (0, rxjs_1.combineLatest)([
            (0, rxjs_1.of)(result),
            result.length
                ? api.derive.democracy.preimages(result.map(({ imageHash }) => imageHash))
                : (0, rxjs_1.of)([])
        ]);
    }), (0, rxjs_1.map)(([infos, images]) => infos.map((info, index) => (0, util_1.objectSpread)({ image: images[index] }, info))));
}
/**
 * @name dispatchQueue
 * @description Retrieves the list of scheduled or pending dispatches in the governance system.
 * @example
 * ```javascript
 * const queue = await api.derive.democracy.dispatchQueue();
 * console.log("Dispatch Queue:", queue);
 * ```
 */
function dispatchQueue(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => (0, util_1.isFunction)(api.query.scheduler?.agenda)
        ? queryScheduler(api)
        : api.query.democracy['dispatchQueue']
            ? queryQueue(api)
            : (0, rxjs_1.of)([]));
}
