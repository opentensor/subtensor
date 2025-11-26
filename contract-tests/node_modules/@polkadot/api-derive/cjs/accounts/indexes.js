"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.indexes = indexes;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
let indicesCache = null;
function queryAccounts(api) {
    return api.query.indices.accounts.entries().pipe((0, rxjs_1.map)((entries) => entries.reduce((indexes, [key, idOpt]) => {
        if (idOpt.isSome) {
            indexes[idOpt.unwrap()[0].toString()] = api.registry.createType('AccountIndex', key.args[0]);
        }
        return indexes;
    }, {})));
}
/**
 * @name indexes
 * @returns Returns all the indexes on the system.
 * @description This is an unwieldly query since it loops through
 * all of the enumsets and returns all of the values found. This could be up to 32k depending
 * on the number of active accounts in the system.
 * @example
 * ```javascript
 * api.derive.accounts.indexes((indexes) => {
 *   console.log('All existing AccountIndexes', indexes);
 * });
 * ```
 */
function indexes(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => indicesCache
        ? (0, rxjs_1.of)(indicesCache)
        : (api.query.indices
            ? queryAccounts(api).pipe((0, rxjs_1.startWith)({}))
            : (0, rxjs_1.of)({})).pipe((0, rxjs_1.map)((indices) => {
            indicesCache = indices;
            return indices;
        })));
}
