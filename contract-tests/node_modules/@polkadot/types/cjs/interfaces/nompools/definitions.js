"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const runtime_js_1 = require("./runtime.js");
exports.default = {
    rpc: {},
    runtime: runtime_js_1.runtime,
    types: {
        NpApiError: {
            _enum: ['MemberNotFound', 'OverflowInPendingRewards']
        },
        NpPoolId: 'u32'
    }
};
