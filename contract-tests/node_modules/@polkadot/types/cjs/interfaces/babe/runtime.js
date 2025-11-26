"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.runtime = void 0;
const V1_V2_SHARED = {
    current_epoch: {
        description: 'Returns information regarding the current epoch.',
        params: [],
        type: 'Epoch'
    },
    current_epoch_start: {
        description: 'Returns the slot that started the current epoch.',
        params: [],
        type: 'Slot'
    },
    generate_key_ownership_proof: {
        description: 'Generates a proof of key ownership for the given authority in the current epoch.',
        params: [
            {
                name: 'slot',
                type: 'Slot'
            },
            {
                name: 'authorityId',
                type: 'AuthorityId'
            }
        ],
        type: 'Option<OpaqueKeyOwnershipProof>'
    },
    next_epoch: {
        description: 'Returns information regarding the next epoch (which was already previously announced).',
        params: [],
        type: 'Epoch'
    },
    submit_report_equivocation_unsigned_extrinsic: {
        description: 'Submits an unsigned extrinsic to report an equivocation.',
        params: [
            {
                name: 'equivocationProof',
                type: 'BabeEquivocationProof'
            },
            {
                name: 'keyOwnerProof',
                type: 'OpaqueKeyOwnershipProof'
            }
        ],
        type: 'Option<Null>'
    }
};
exports.runtime = {
    BabeApi: [
        {
            methods: {
                configuration: {
                    description: 'Return the genesis configuration for BABE. The configuration is only read on genesis.',
                    params: [],
                    type: 'BabeGenesisConfiguration'
                },
                ...V1_V2_SHARED
            },
            version: 2
        },
        {
            methods: {
                configuration: {
                    description: 'Return the configuration for BABE. Version 1.',
                    params: [],
                    type: 'BabeGenesisConfigurationV1'
                },
                ...V1_V2_SHARED
            },
            version: 1
        }
    ]
};
