import { rpc } from './rpc.js';
import { runtime } from './runtime.js';
export default {
    rpc,
    runtime,
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
