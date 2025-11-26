"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.memo = memo;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const drr_js_1 = require("./drr.js");
/** @internal */
function memo(instanceId, inner) {
    const options = { getInstanceId: () => instanceId };
    const cached = (0, util_1.memoize)((...params) => new rxjs_1.Observable((observer) => {
        const subscription = inner(...params).subscribe(observer);
        return () => {
            cached.unmemoize(...params);
            subscription.unsubscribe();
        };
    }).pipe((0, drr_js_1.drr)()), options);
    return cached;
}
