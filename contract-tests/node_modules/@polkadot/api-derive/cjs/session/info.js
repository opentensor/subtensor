"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.info = info;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
/**
 * @name info
 * @description Retrieves all the session and era query and calculates specific values on it as the length of the session and eras.
 * @example
 * ```javascript
 * api.derive.session.info((info) => {
 *   console.log(`Session info ${JSON.stringify(info)}`);
 * });
 * ```
 */
function info(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => api.derive.session.indexes().pipe((0, rxjs_1.map)((indexes) => {
        const sessionLength = api.consts?.babe?.epochDuration || api.registry.createType('u64', 1);
        const sessionsPerEra = api.consts?.staking?.sessionsPerEra || api.registry.createType('SessionIndex', 1);
        return (0, util_1.objectSpread)({
            eraLength: api.registry.createType('BlockNumber', sessionsPerEra.mul(sessionLength)),
            isEpoch: !!api.query.babe,
            sessionLength,
            sessionsPerEra
        }, indexes);
    })));
}
