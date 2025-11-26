"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.drr = drr;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const refCountDelay_js_1 = require("./refCountDelay.js");
function CMP(a, b) {
    return (0, util_1.stringify)({ t: a }) === (0, util_1.stringify)({ t: b });
}
function ERR(error) {
    throw error;
}
function NOOP() {
    // empty
}
/**
 * Shorthand for distinctUntilChanged(), publishReplay(1) and refCount().
 *
 * @ignore
 * @internal
 */
function drr({ delay, skipChange = false, skipTimeout = false } = {}) {
    return (source$) => source$.pipe((0, rxjs_1.catchError)(ERR), skipChange
        ? (0, rxjs_1.tap)(NOOP)
        : (0, rxjs_1.distinctUntilChanged)(CMP), 
    // eslint-disable-next-line deprecation/deprecation
    (0, rxjs_1.publishReplay)(1), skipTimeout
        // eslint-disable-next-line deprecation/deprecation
        ? (0, rxjs_1.refCount)()
        : (0, refCountDelay_js_1.refCountDelay)(delay));
}
