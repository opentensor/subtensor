"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.info = info;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
function retrieveNick(api, accountId) {
    return (accountId && api.query['nicks']?.['nameOf']
        ? api.query['nicks']['nameOf'](accountId)
        : (0, rxjs_1.of)(undefined)).pipe((0, rxjs_1.map)((nameOf) => nameOf?.isSome
        ? (0, util_1.u8aToString)(nameOf.unwrap()[0]).substring(0, api.consts['nicks']['maxLength'].toNumber())
        : undefined));
}
/**
 * @name info
 * @description Returns aux. info with regards to an account, current that includes the accountId, accountIndex, identity and nickname
 * @param {(AccountIndex | AccountId | Address | Uint8Array | string | null)} address An accounts in different formats.
 * @example
 * ```javascript
 * const ALICE = "13AU";
 * const info = await api.derive.accounts.info(ALICE);
 * console.log(
 *   "Account Info: ",
 *   Object.keys(info).map((key) => `${key}: ${info[key]}`)
 * );
 * ```
 */
function info(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (address) => api.derive.accounts.idAndIndex(address).pipe((0, rxjs_1.switchMap)(([accountId, accountIndex]) => (0, rxjs_1.combineLatest)([
        (0, rxjs_1.of)({ accountId, accountIndex }),
        api.derive.accounts.identity(accountId),
        retrieveNick(api, accountId)
    ])), (0, rxjs_1.map)(([{ accountId, accountIndex }, identity, nickname]) => ({
        accountId, accountIndex, identity, nickname
    }))));
}
