"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.versioned = void 0;
const types_create_1 = require("@polkadot/types-create");
/* eslint-disable sort-keys */
const sharedTypes = {
    DispatchErrorModule: 'DispatchErrorModuleU8',
    FullIdentification: '()', // No staking, only session (as per config)
    Keys: 'SessionKeys7B',
    Weight: 'WeightV1'
};
exports.versioned = [
    {
        minmax: [0, 200],
        types: {
            ...sharedTypes,
            AccountInfo: 'AccountInfoWithDualRefCount',
            Address: 'AccountId',
            LookupSource: 'AccountId'
        }
    },
    {
        minmax: [201, 214],
        types: {
            ...sharedTypes,
            AccountInfo: 'AccountInfoWithDualRefCount'
        }
    },
    {
        minmax: [215, 228],
        types: {
            ...sharedTypes,
            Keys: 'SessionKeys6'
        }
    },
    {
        minmax: [229, 9099],
        types: {
            ...sharedTypes,
            ...(0, types_create_1.mapXcmTypes)('V0')
        }
    },
    {
        minmax: [9100, 9105],
        types: {
            ...sharedTypes,
            ...(0, types_create_1.mapXcmTypes)('V1')
        }
    },
    {
        // metadata v14
        minmax: [9106, undefined],
        types: {
            Weight: 'WeightV1'
        }
    }
    // ,
    // {
    //   // weight v2 introduction
    //   minmax: [9300, undefined],
    //   types: {
    //     Weight: 'WeightV2'
    //   }
    // }
];
