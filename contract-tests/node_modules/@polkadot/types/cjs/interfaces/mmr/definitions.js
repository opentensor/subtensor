"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const rpc_js_1 = require("./rpc.js");
const runtime_js_1 = require("./runtime.js");
exports.default = {
    rpc: rpc_js_1.rpc,
    runtime: runtime_js_1.runtime,
    types: {
        MmrBatchProof: {
            leafIndices: 'Vec<MmrLeafIndex>',
            leafCount: 'MmrNodeIndex',
            items: 'Vec<Hash>'
        },
        MmrEncodableOpaqueLeaf: 'Bytes',
        MmrError: {
            _enum: ['InvalidNumericOp', 'Push', 'GetRoot', 'Commit', 'GenerateProof', 'Verify', 'LeafNotFound', ' PalletNotIncluded', 'InvalidLeafIndex', 'InvalidBestKnownBlock']
        },
        MmrHash: 'Hash',
        MmrLeafBatchProof: {
            blockHash: 'BlockHash',
            leaves: 'Bytes',
            proof: 'Bytes'
        },
        MmrLeafIndex: 'u64',
        MmrLeafProof: {
            blockHash: 'BlockHash',
            leaf: 'Bytes',
            proof: 'Bytes'
        },
        MmrNodeIndex: 'u64',
        MmrProof: {
            leafIndex: 'MmrLeafIndex',
            leafCount: 'MmrNodeIndex',
            items: 'Vec<Hash>'
        }
    }
};
