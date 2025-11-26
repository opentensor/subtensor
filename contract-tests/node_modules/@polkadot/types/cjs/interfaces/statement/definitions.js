"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const runtime_js_1 = require("./runtime.js");
exports.default = {
    rpc: {},
    runtime: runtime_js_1.runtime,
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
