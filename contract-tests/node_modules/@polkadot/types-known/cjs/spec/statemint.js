"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.versioned = void 0;
const types_create_1 = require("@polkadot/types-create");
const sharedTypes = {
    DispatchErrorModule: 'DispatchErrorModuleU8',
    TAssetBalance: 'u128',
    ProxyType: {
        _enum: [
            'Any',
            'NonTransfer',
            'CancelProxy',
            'Assets',
            'AssetOwner',
            'AssetManager',
            'Staking'
        ]
    },
    Weight: 'WeightV1'
};
exports.versioned = [
    {
        minmax: [0, 3],
        types: {
            // Enum was modified mid-flight -
            // https://github.com/paritytech/substrate/pull/10382/files#diff-e4e016b33a82268b6208dc974eea841bad47597865a749fee2f937eb6fdf67b4R498
            DispatchError: 'DispatchErrorPre6First',
            ...sharedTypes,
            ...(0, types_create_1.mapXcmTypes)('V0')
        }
    },
    {
        minmax: [4, 5],
        types: {
            // As above, see https://github.com/polkadot-js/api/issues/5301
            DispatchError: 'DispatchErrorPre6First',
            ...sharedTypes,
            ...(0, types_create_1.mapXcmTypes)('V1')
        }
    },
    {
        // metadata V14
        minmax: [500, 1001003],
        types: {
            Weight: 'WeightV1',
            TAssetConversion: 'Option<AssetId>'
        }
    },
    // ref: https://github.com/polkadot-js/api/pull/6208
    // The following patches an issue where polkadot asset hub block 9686187, can't be decoding using xcm v5
    // mappings.
    {
        minmax: [1002000, undefined],
        types: {
            Weight: 'WeightV1',
            ...(0, types_create_1.mapXcmTypes)('V4')
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
