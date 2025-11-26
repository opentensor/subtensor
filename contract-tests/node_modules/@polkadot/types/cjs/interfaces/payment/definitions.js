"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const rpc_js_1 = require("./rpc.js");
const runtime_js_1 = require("./runtime.js");
exports.default = {
    rpc: rpc_js_1.rpc,
    runtime: runtime_js_1.runtime,
    types: {
        FeeDetails: {
            inclusionFee: 'Option<InclusionFee>'
            // skipped in serde
            // tip: 'Balance'
        },
        InclusionFee: {
            baseFee: 'Balance',
            lenFee: 'Balance',
            adjustedWeightFee: 'Balance'
        },
        RuntimeDispatchInfo: {
            weight: 'Weight',
            class: 'DispatchClass',
            partialFee: 'Balance'
        },
        RuntimeDispatchInfoV1: {
            weight: 'WeightV1',
            class: 'DispatchClass',
            partialFee: 'Balance'
        },
        RuntimeDispatchInfoV2: {
            weight: 'WeightV2',
            class: 'DispatchClass',
            partialFee: 'Balance'
        }
    }
};
