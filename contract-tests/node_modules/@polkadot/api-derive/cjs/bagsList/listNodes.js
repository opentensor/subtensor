"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.listNodes = listNodes;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
const util_js_1 = require("./util.js");
function traverseLinks(api, head) {
    const subject = new rxjs_1.BehaviorSubject(head);
    const query = (0, util_js_1.getQueryInterface)(api);
    return subject.pipe((0, rxjs_1.switchMap)((account) => query.listNodes(account)), (0, rxjs_1.tap)((node) => {
        (0, util_1.nextTick)(() => {
            node.isSome && node.value.next.isSome
                ? subject.next(node.unwrap().next.unwrap())
                : subject.complete();
        });
    }), (0, rxjs_1.toArray)(), // toArray since we want to startSubject to be completed
    (0, rxjs_1.map)((all) => all.map((o) => o.unwrap())));
}
/**
 * @name listNodes
 * @param {(PalletBagsListListBag | null)} bag A reference to a specific bag in the BagsList pallet.
 * @description Retrieves the list of nodes (accounts) contained in a specific bag within the BagsList pallet.
 */
function listNodes(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (bag) => bag && bag.head.isSome
        ? traverseLinks(api, bag.head.unwrap())
        : (0, rxjs_1.of)([]));
}
