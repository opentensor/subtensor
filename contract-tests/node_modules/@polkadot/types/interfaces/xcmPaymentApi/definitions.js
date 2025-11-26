import { runtime } from './runtime.js';
export default {
    rpc: {},
    runtime,
    types: {
        XcmPaymentApiError: {
            _enum: [
                'Unimplemented',
                'VersionedConversionFailed',
                'WeightNotComputable',
                'UnhandledXcmVersion',
                'AssetNotFound'
            ]
        }
    }
};
