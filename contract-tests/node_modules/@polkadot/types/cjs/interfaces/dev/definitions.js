"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const rpc_js_1 = require("./rpc.js");
exports.default = {
    rpc: rpc_js_1.rpc,
    types: {
        BlockStats: {
            witnessLen: 'u64',
            witnessCompactLen: 'u64',
            blockLen: 'u64',
            blockNumExtrinsics: 'u64'
        }
    }
};
