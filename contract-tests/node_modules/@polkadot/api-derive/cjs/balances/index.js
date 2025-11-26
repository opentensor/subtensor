"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.votingBalance = exports.all = void 0;
const tslib_1 = require("tslib");
const all_js_1 = require("./all.js");
Object.defineProperty(exports, "all", { enumerable: true, get: function () { return all_js_1.all; } });
tslib_1.__exportStar(require("./account.js"), exports);
tslib_1.__exportStar(require("./votingBalances.js"), exports);
/**
 * @name votingBalance
 * @param {( AccountId | string )} address An accounts Id in different formats.
 * @returns An object containing the results of various balance queries
 * @example
 * <BR>
 *
 * ```javascript
 * const ALICE = 'F7Hs';
 *
 * api.derive.balances.votingBalance(ALICE, ({ accountId, lockedBalance }) => {
 *   console.log(`The account ${accountId} has a locked balance ${lockedBalance} units.`);
 * });
 * ```
 */
const votingBalance = all_js_1.all;
exports.votingBalance = votingBalance;
