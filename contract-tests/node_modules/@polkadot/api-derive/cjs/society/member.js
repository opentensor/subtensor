"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.member = member;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
/**
 * @name member
 * @description Get the member info for a society.
 * @param { AccountId } accountId
 * @example
 * ```javascript
 * const member = await api.derive.society.member(ALICE);
 * console.log(member);
 * ```
 */
function member(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (accountId) => api.derive.society._members([accountId]).pipe((0, rxjs_1.map)(([result]) => result)));
}
