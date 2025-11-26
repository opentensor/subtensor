"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const runtime_js_1 = require("./runtime.js");
exports.default = {
    rpc: {},
    runtime: runtime_js_1.runtime,
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
