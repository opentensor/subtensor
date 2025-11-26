import { runtime } from './runtime.js';
export default {
    rpc: {},
    runtime,
    types: {
        TransactionSource: {
            _enum: ['InBlock', 'Local', 'External']
        },
        TransactionValidity: 'Result<ValidTransaction, TransactionValidityError>',
        ValidTransaction: {
            priority: 'TransactionPriority',
            requires: 'Vec<TransactionTag>',
            provides: 'Vec<TransactionTag>',
            longevity: 'TransactionLongevity',
            propagate: 'bool'
        }
    }
};
