"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const runtime_js_1 = require("./runtime.js");
exports.default = {
    rpc: {},
    runtime: runtime_js_1.runtime,
    types: {
        CheckInherentsResult: {
            okay: 'bool',
            fatalError: 'bool',
            errors: 'InherentData'
        },
        InherentData: {
            data: 'BTreeMap<InherentIdentifier, Bytes>'
        },
        InherentIdentifier: '[u8; 8]'
    }
};
