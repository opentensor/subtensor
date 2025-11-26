import { rpc } from './rpc.js';
export default {
    rpc,
    types: {
        BlockStats: {
            witnessLen: 'u64',
            witnessCompactLen: 'u64',
            blockLen: 'u64',
            blockNumExtrinsics: 'u64'
        }
    }
};
