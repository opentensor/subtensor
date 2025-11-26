import { runtime } from './runtime.js';
export default {
    rpc: {},
    runtime,
    types: {
        Error: {
            _enum: ['Unsupported', 'VersionedConversionFailed']
        }
    }
};
