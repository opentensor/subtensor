"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.filterEvents = filterEvents;
const util_1 = require("@polkadot/util");
const logging_js_1 = require("./logging.js");
function filterEvents(txHash, { block: { extrinsics, header } }, allEvents, status) {
    // extrinsics to hashes
    for (const [txIndex, x] of extrinsics.entries()) {
        if (x.hash.eq(txHash)) {
            return {
                blockNumber: (0, util_1.isCompact)(header.number) ? header.number.unwrap() : header.number,
                events: allEvents.filter(({ phase }) => phase.isApplyExtrinsic &&
                    phase.asApplyExtrinsic.eqn(txIndex)),
                txIndex
            };
        }
    }
    // if we do get the block after finalized, it _should_ be there
    // only warn on filtering with isInBlock (finalization finalizes after)
    if (status.isInBlock) {
        const allHashes = extrinsics.map((x) => x.hash.toHex());
        logging_js_1.l.warn(`block ${header.hash.toHex()}: Unable to find extrinsic ${txHash.toHex()} inside ${allHashes.join(', ')}`);
    }
    return {};
}
