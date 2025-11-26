"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getQueryInterface = getQueryInterface;
function getQueryInterface(api) {
    return (
    // latest substrate & polkadot
    api.query.voterList ||
        // previous substrate
        api.query['voterBagsList'] ||
        api.query['bagsList']);
}
