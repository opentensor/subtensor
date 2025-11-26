"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.rpc = void 0;
exports.rpc = {
    getBlockStats: {
        description: 'Reexecute the specified `block_hash` and gather statistics while doing so',
        isUnsafe: true,
        params: [
            {
                isHistoric: true,
                name: 'at',
                type: 'Hash'
            }
        ],
        type: 'Option<BlockStats>'
    }
};
