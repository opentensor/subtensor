"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.signingInfo = signingInfo;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
const constants_js_1 = require("./constants.js");
function latestNonce(api, address) {
    return api.derive.balances.account(address).pipe((0, rxjs_1.map)(({ accountNonce }) => accountNonce));
}
function nextNonce(api, address) {
    if (api.call.accountNonceApi) {
        return api.call.accountNonceApi.accountNonce(address);
    }
    else {
        return api.rpc.system?.accountNextIndex
            ? api.rpc.system.accountNextIndex(address)
            : latestNonce(api, address);
    }
}
function signingHeader(api) {
    return (0, rxjs_1.combineLatest)([
        api.rpc.chain.getHeader().pipe((0, rxjs_1.switchMap)((header) => 
        // check for chains at genesis (until block 1 is produced, e.g. 6s), since
        // we do need to allow transactions at chain start (also dev/seal chains)
        header.parentHash.isEmpty
            ? (0, rxjs_1.of)(header)
            // in the case of the current block, we use the parent to minimize the
            // impact of forks on the system, but not completely remove it
            : api.rpc.chain.getHeader(header.parentHash).pipe((0, rxjs_1.catchError)(() => (0, rxjs_1.of)(header))))),
        api.rpc.chain.getFinalizedHead().pipe((0, rxjs_1.switchMap)((hash) => api.rpc.chain.getHeader(hash).pipe((0, rxjs_1.catchError)(() => (0, rxjs_1.of)(null)))))
    ]).pipe((0, rxjs_1.map)(([current, finalized]) => 
    // determine the hash to use, current when lag > max, else finalized
    !finalized || (0, index_js_1.unwrapBlockNumber)(current).sub((0, index_js_1.unwrapBlockNumber)(finalized)).gt(constants_js_1.MAX_FINALITY_LAG)
        ? current
        : finalized));
}
function babeOrAuraPeriod(api) {
    const period = api.consts.babe?.expectedBlockTime ||
        // this will be present ones https://github.com/paritytech/polkadot-sdk/pull/3732 is merged
        // eslint-disable-next-line
        api.consts['aura']?.['slotDuration'] ||
        api.consts.timestamp?.minimumPeriod.muln(2);
    return period && period.isZero && !period.isZero() ? period : undefined;
}
/**
 * @name signingInfo
 * @description Retrieves signing-related information for an account, including the nonce, block header, and mortal length.
 * @param {string} address The account address for which signing information is needed.
 * @param { BN | bigint | Uint8Array | number | string } nonce? (Optional) The nonce to use. If `undefined`, the latest nonce is retrieved.
 * @param { IExtrinsicEra | number } era? (Optional) The transaction era.
 * @example
 * ```javascript
 * const info = await api.derive.tx.signingInfo(
 *   "14mM9FRDDtwSYicjNxSvMfQkap8o4m9zHq7hNW4JpbSL4PPU"
 * );
 * console.log(info);
 * ```
 */
function signingInfo(_instanceId, api) {
    // no memo, we want to do this fresh on each run
    return (address, nonce, era) => (0, rxjs_1.combineLatest)([
        // retrieve nonce if none was specified
        (0, util_1.isUndefined)(nonce)
            ? latestNonce(api, address)
            : nonce === -1
                ? nextNonce(api, address)
                : (0, rxjs_1.of)(api.registry.createType('Index', nonce)),
        // if no era (create) or era > 0 (mortal), do block retrieval
        ((0, util_1.isUndefined)(era) || ((0, util_1.isNumber)(era) && era > 0))
            ? signingHeader(api)
            : (0, rxjs_1.of)(null)
    ]).pipe((0, rxjs_1.map)(([nonce, header]) => ({
        header,
        mortalLength: Math.min(api.consts.system?.blockHashCount?.toNumber() || constants_js_1.FALLBACK_MAX_HASH_COUNT, constants_js_1.MORTAL_PERIOD
            .div(babeOrAuraPeriod(api) || constants_js_1.FALLBACK_PERIOD)
            .iadd(constants_js_1.MAX_FINALITY_LAG)
            .toNumber()),
        nonce
    })));
}
