import { runtime } from './runtime.js';
export default {
    rpc: {},
    runtime,
    types: {
        NpApiError: {
            _enum: ['MemberNotFound', 'OverflowInPendingRewards']
        },
        NpPoolId: 'u32'
    }
};
