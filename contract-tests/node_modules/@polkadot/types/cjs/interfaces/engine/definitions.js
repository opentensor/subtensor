"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const rpc_js_1 = require("./rpc.js");
exports.default = {
    rpc: rpc_js_1.rpc,
    types: {
        CreatedBlock: {
            _alias: {
                blockHash: 'hash'
            },
            blockHash: 'BlockHash',
            aux: 'ImportedAux'
        },
        ImportedAux: {
            headerOnly: 'bool',
            clearJustificationRequests: 'bool',
            needsJustification: 'bool',
            badJustification: 'bool',
            needsFinalityProof: 'bool',
            isNewBest: 'bool'
        }
    }
};
