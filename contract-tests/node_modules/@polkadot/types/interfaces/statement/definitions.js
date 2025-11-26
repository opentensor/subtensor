import { runtime } from './runtime.js';
export default {
    rpc: {},
    runtime,
    types: {
        StatementStoreStatementSource: {
            _enum: ['Chain', 'Network', 'Local']
        },
        StatementStoreValidStatement: {
            maxCount: 'u32',
            maxSize: 'u32'
        },
        StatementStoreInvalidStatement: {
            _enum: ['BadProof', 'NoProof', 'InternalError']
        }
    }
};
