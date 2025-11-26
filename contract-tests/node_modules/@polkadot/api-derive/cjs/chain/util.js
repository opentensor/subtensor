"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createBlockNumberDerive = createBlockNumberDerive;
exports.getAuthorDetails = getAuthorDetails;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
function createBlockNumberDerive(fn) {
    return (instanceId, api) => (0, index_js_1.memo)(instanceId, () => fn(api).pipe((0, rxjs_1.map)(index_js_1.unwrapBlockNumber)));
}
/** @internal */
function getAuthorDetailsWithAt(header, queryAt) {
    const validators = queryAt.session?.validators
        ? queryAt.session.validators()
        : (0, rxjs_1.of)(null);
    // nimbus consensus stores the session key of the block author in header logs
    const { logs: [log] } = header.digest;
    const loggedAuthor = (log && ((log.isConsensus && log.asConsensus[0].isNimbus && log.asConsensus[1]) ||
        (log.isPreRuntime && log.asPreRuntime[0].isNimbus && log.asPreRuntime[1])));
    if (loggedAuthor) {
        // use the author mapping pallet, if available (ie: moonbeam, moonriver), to map session (nimbus) key to author (collator/validator) key
        if (queryAt['authorMapping']?.['mappingWithDeposit']) {
            return (0, rxjs_1.combineLatest)([
                (0, rxjs_1.of)(header),
                validators,
                queryAt['authorMapping']['mappingWithDeposit'](loggedAuthor).pipe((0, rxjs_1.map)((o) => o.unwrapOr({ account: null }).account))
            ]);
        }
        // fall back to session and parachain staking pallets, if available (ie: manta, calamari), to map session (nimbus) key to author (collator) key
        if (queryAt['parachainStaking']?.['selectedCandidates'] && queryAt.session?.nextKeys) {
            const loggedHex = loggedAuthor.toHex();
            return (0, rxjs_1.combineLatest)([
                (0, rxjs_1.of)(header),
                validators,
                queryAt['parachainStaking']['selectedCandidates']().pipe((0, rxjs_1.mergeMap)((selectedCandidates) => (0, rxjs_1.combineLatest)([
                    (0, rxjs_1.of)(selectedCandidates),
                    queryAt.session.nextKeys.multi(selectedCandidates).pipe((0, rxjs_1.map)((nextKeys) => nextKeys.findIndex((o) => o.unwrapOrDefault().nimbus.toHex() === loggedHex)))
                ])), (0, rxjs_1.map)(([selectedCandidates, index]) => index === -1
                    ? null
                    : selectedCandidates[index]))
            ]);
        }
    }
    // normal operation, non-mapping
    return (0, rxjs_1.combineLatest)([
        (0, rxjs_1.of)(header),
        validators,
        (0, rxjs_1.of)(null)
    ]);
}
function getAuthorDetails(api, header, blockHash) {
    // For on-chain state, we need to retrieve it as per the start
    // of the block being constructed, i.e. session validators would
    // be at the point of the block construction, not when all operations
    // has been supplied.
    //
    // However for the first block (no parentHash available), we would
    // just use the as-is
    return api.queryAt(header.parentHash.isEmpty
        ? blockHash || header.hash
        : header.parentHash).pipe((0, rxjs_1.switchMap)((queryAt) => getAuthorDetailsWithAt(header, queryAt)));
}
