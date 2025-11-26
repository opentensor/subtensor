"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.AllHashers = void 0;
const hashers_js_1 = require("./hashers.js");
Object.defineProperty(exports, "AllHashers", { enumerable: true, get: function () { return hashers_js_1.AllHashers; } });
const runtime_js_1 = require("./runtime.js");
const v9_js_1 = require("./v9.js");
const v10_js_1 = require("./v10.js");
const v11_js_1 = require("./v11.js");
const v12_js_1 = require("./v12.js");
const v13_js_1 = require("./v13.js");
const v14_js_1 = require("./v14.js");
const v15_js_1 = require("./v15.js");
const v16_js_1 = require("./v16.js");
exports.default = {
    rpc: {},
    runtime: runtime_js_1.runtime,
    types: {
        // all known
        ...v9_js_1.v9,
        ...v10_js_1.v10,
        ...v11_js_1.v11,
        ...v12_js_1.v12,
        ...v13_js_1.v13,
        ...v14_js_1.v14,
        ...v15_js_1.v15,
        ...v16_js_1.v16,
        // latest mappings
        // NOTE: For v15, we only added the runtime defintions,
        // hence latest for most pointing to the previous V14
        ErrorMetadataLatest: 'ErrorMetadataV14',
        EventMetadataLatest: 'EventMetadataV14',
        ExtrinsicMetadataLatest: 'ExtrinsicMetadataV16',
        FunctionArgumentMetadataLatest: 'FunctionArgumentMetadataV14',
        FunctionMetadataLatest: 'FunctionMetadataV14',
        MetadataLatest: 'MetadataV16',
        PalletCallMetadataLatest: 'PalletCallMetadataV16',
        PalletConstantMetadataLatest: 'PalletConstantMetadataV16',
        PalletErrorMetadataLatest: 'PalletErrorMetadataV16',
        PalletEventMetadataLatest: 'PalletEventMetadataV16',
        PalletMetadataLatest: 'PalletMetadataV16',
        PalletStorageMetadataLatest: 'PalletStorageMetadataV16',
        PortableType: 'PortableTypeV14',
        RuntimeApiMetadataLatest: 'RuntimeApiMetadataV16',
        SignedExtensionMetadataLatest: 'SignedExtensionMetadataV14',
        TransactionExtensionMetadataLatest: 'TransactionExtensionMetadataV16',
        StorageEntryMetadataLatest: 'StorageEntryMetadataV16',
        StorageEntryModifierLatest: 'StorageEntryModifierV14',
        StorageEntryTypeLatest: 'StorageEntryTypeV14',
        StorageHasher: 'StorageHasherV14',
        // additional types
        OpaqueMetadata: 'Opaque<Bytes>',
        // the enum containing all the mappings
        MetadataAll: {
            _enum: {
                V0: 'DoNotConstruct<MetadataV0>',
                V1: 'DoNotConstruct<MetadataV1>',
                V2: 'DoNotConstruct<MetadataV2>',
                V3: 'DoNotConstruct<MetadataV3>',
                V4: 'DoNotConstruct<MetadataV4>',
                V5: 'DoNotConstruct<MetadataV5>',
                V6: 'DoNotConstruct<MetadataV6>',
                V7: 'DoNotConstruct<MetadataV7>',
                V8: 'DoNotConstruct<MetadataV8>',
                // First version on Kusama in V9, dropping will be problematic
                V9: 'MetadataV9',
                V10: 'MetadataV10',
                V11: 'MetadataV11',
                V12: 'MetadataV12',
                V13: 'MetadataV13',
                V14: 'MetadataV14',
                V15: 'MetadataV15',
                V16: 'MetadataV16'
            }
        }
    }
};
