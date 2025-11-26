import { rpc } from './rpc.js';
export default {
    rpc,
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
