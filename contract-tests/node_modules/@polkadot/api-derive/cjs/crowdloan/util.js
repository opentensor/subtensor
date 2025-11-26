"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.extractContributed = extractContributed;
function extractContributed(paraId, events) {
    const added = [];
    const removed = [];
    return events
        .filter(({ event: { data: [, eventParaId], method, section } }) => section === 'crowdloan' &&
        ['Contributed', 'Withdrew'].includes(method) &&
        eventParaId.eq(paraId))
        .reduce((result, { event: { data: [accountId], method } }) => {
        if (method === 'Contributed') {
            result.added.push(accountId.toHex());
        }
        else {
            result.removed.push(accountId.toHex());
        }
        return result;
    }, { added, blockHash: events.createdAtHash?.toHex() || '-', removed });
}
