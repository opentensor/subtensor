"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.rpc = void 0;
exports.rpc = {
    createBlock: {
        description: 'Instructs the manual-seal authorship task to create a new block',
        params: [
            {
                name: 'createEmpty',
                type: 'bool'
            },
            {
                name: 'finalize',
                type: 'bool'
            },
            {
                isOptional: true,
                name: 'parentHash',
                type: 'BlockHash'
            }
        ],
        type: 'CreatedBlock'
    },
    finalizeBlock: {
        description: 'Instructs the manual-seal authorship task to finalize a block',
        params: [
            {
                name: 'hash',
                type: 'BlockHash'
            },
            {
                isOptional: true,
                name: 'justification',
                type: 'Justification'
            }
        ],
        type: 'bool'
    }
};
