import { rpc } from './rpc.js';
import { runtime } from './runtime.js';
export default {
    rpc,
    runtime,
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
