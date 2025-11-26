import { runtime } from './runtime.js';
export default {
    rpc: {},
    runtime,
    types: {
        CheckInherentsResult: {
            okay: 'bool',
            fatalError: 'bool',
            errors: 'InherentData'
        },
        InherentData: {
            data: 'BTreeMap<InherentIdentifier, Bytes>'
        },
        InherentIdentifier: '[u8; 8]'
    }
};
