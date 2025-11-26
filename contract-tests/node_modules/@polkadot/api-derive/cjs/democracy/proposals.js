"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.proposals = proposals;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
const util_js_1 = require("./util.js");
function isNewDepositors(depositors) {
    // Detect balance...
    return (0, util_1.isFunction)(depositors[1].mul);
}
function parse([proposals, images, optDepositors]) {
    return proposals
        .filter(([, , proposer], index) => !!(optDepositors[index]?.isSome) && !proposer.isEmpty)
        .map(([index, hash, proposer], proposalIndex) => {
        const depositors = optDepositors[proposalIndex].unwrap();
        return (0, util_1.objectSpread)({
            image: images[proposalIndex],
            imageHash: (0, util_js_1.getImageHashBounded)(hash),
            index,
            proposer
        }, isNewDepositors(depositors)
            ? { balance: depositors[1], seconds: depositors[0] }
            : { balance: depositors[0], seconds: depositors[1] });
    });
}
/**
 * @name proposals
 * @description Retrieves the list of active public proposals in the democracy module, along with their associated preimage data and deposit information.
 * @example
 * ```javascript
 * const proposals = await api.derive.democracy.proposals();
 * console.log("proposals:", proposals);
 * ```
 */
function proposals(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => (0, util_1.isFunction)(api.query.democracy?.publicProps)
        ? api.query.democracy.publicProps().pipe((0, rxjs_1.switchMap)((proposals) => proposals.length
            ? (0, rxjs_1.combineLatest)([
                (0, rxjs_1.of)(proposals),
                api.derive.democracy.preimages(proposals.map(([, hash]) => hash)),
                api.query.democracy.depositOf.multi(proposals.map(([index]) => index))
            ])
            : (0, rxjs_1.of)([[], [], []])), (0, rxjs_1.map)(parse))
        : (0, rxjs_1.of)([]));
}
