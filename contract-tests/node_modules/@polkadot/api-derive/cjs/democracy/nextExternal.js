"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.nextExternal = nextExternal;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
const util_js_1 = require("./util.js");
function withImage(api, nextOpt) {
    if (nextOpt.isNone) {
        return (0, rxjs_1.of)(null);
    }
    const [hash, threshold] = nextOpt.unwrap();
    return api.derive.democracy.preimage(hash).pipe((0, rxjs_1.map)((image) => ({
        image,
        imageHash: (0, util_js_1.getImageHashBounded)(hash),
        threshold
    })));
}
/**
 * @name nextExternal
 * @description Retrieves the next external proposal that is scheduled for a referendum.
 * @example
 * ```javascript
 * const nextExternal = await api.derive.democracy.nextExternal();
 * console.log("Next external proposal:", nextExternal);
 * ```
 */
function nextExternal(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => api.query.democracy?.nextExternal
        ? api.query.democracy.nextExternal().pipe((0, rxjs_1.switchMap)((nextOpt) => withImage(api, nextOpt)))
        : (0, rxjs_1.of)(null));
}
