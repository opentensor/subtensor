(function (global, factory) {
    typeof exports === 'object' && typeof module !== 'undefined' ? factory(exports, require('@polkadot/util'), require('@polkadot/util-crypto')) :
    typeof define === 'function' && define.amd ? define(['exports', '@polkadot/util', '@polkadot/util-crypto'], factory) :
    (global = typeof globalThis !== 'undefined' ? globalThis : global || self, factory(global.polkadotTypes = {}, global.polkadotUtil, global.polkadotUtilCrypto));
})(this, (function (exports, util, utilCrypto) { 'use strict';

    const global = typeof globalThis !== "undefined" ? globalThis : typeof self !== "undefined" ? self : window;

    var _documentCurrentScript = typeof document !== 'undefined' ? document.currentScript : null;
    const AllHashers = {
        Blake2_128: null,
        Blake2_256: null,
        Blake2_128Concat: null,
        Twox128: null,
        Twox256: null,
        Twox64Concat: null,
        Identity: null
    };

    const META_V1_TO_V2 = {
        metadata: {
            description: 'Returns the metadata of a runtime',
            params: [],
            type: 'OpaqueMetadata'
        }
    };
    const runtime$z = {
        Metadata: [
            {
                methods: {
                    metadata_at_version: {
                        description: 'Returns the metadata at a given version.',
                        params: [
                            {
                                name: 'version',
                                type: 'u32'
                            }
                        ],
                        type: 'Option<OpaqueMetadata>'
                    },
                    metadata_versions: {
                        description: 'Returns the supported metadata versions.',
                        params: [],
                        type: 'Vec<u32>'
                    },
                    ...META_V1_TO_V2
                },
                version: 2
            },
            {
                methods: {
                    ...META_V1_TO_V2
                },
                version: 1
            }
        ]
    };

    const v9 = {
        ErrorMetadataV9: {
            name: 'Text',
            docs: 'Vec<Text>'
        },
        EventMetadataV9: {
            name: 'Text',
            args: 'Vec<Type>',
            docs: 'Vec<Text>'
        },
        FunctionArgumentMetadataV9: {
            name: 'Text',
            type: 'Type'
        },
        FunctionMetadataV9: {
            name: 'Text',
            args: 'Vec<FunctionArgumentMetadataV9>',
            docs: 'Vec<Text>'
        },
        MetadataV9: {
            modules: 'Vec<ModuleMetadataV9>'
        },
        ModuleConstantMetadataV9: {
            name: 'Text',
            type: 'Type',
            value: 'Bytes',
            docs: 'Vec<Text>'
        },
        ModuleMetadataV9: {
            name: 'Text',
            storage: 'Option<StorageMetadataV9>',
            calls: 'Option<Vec<FunctionMetadataV9>>',
            events: 'Option<Vec<EventMetadataV9>>',
            constants: 'Vec<ModuleConstantMetadataV9>',
            errors: 'Vec<ErrorMetadataV9>'
        },
        StorageEntryMetadataV9: {
            name: 'Text',
            modifier: 'StorageEntryModifierV9',
            type: 'StorageEntryTypeV9',
            fallback: 'Bytes',
            docs: 'Vec<Text>'
        },
        StorageEntryModifierV9: {
            _enum: ['Optional', 'Default', 'Required']
        },
        StorageEntryTypeV9: {
            _enum: {
                Plain: 'Type',
                Map: {
                    hasher: 'StorageHasherV9',
                    key: 'Type',
                    value: 'Type',
                    linked: 'bool'
                },
                DoubleMap: {
                    hasher: 'StorageHasherV9',
                    key1: 'Type',
                    key2: 'Type',
                    value: 'Type',
                    key2Hasher: 'StorageHasherV9'
                }
            }
        },
        StorageHasherV9: {
            _enum: {
                Blake2_128: null,
                Blake2_256: null,
                Twox128: null,
                Twox256: null,
                Twox64Concat: null
            }
        },
        StorageMetadataV9: {
            prefix: 'Text',
            items: 'Vec<StorageEntryMetadataV9>'
        }
    };

    const v10 = {
        ErrorMetadataV10: 'ErrorMetadataV9',
        EventMetadataV10: 'EventMetadataV9',
        FunctionArgumentMetadataV10: 'FunctionArgumentMetadataV9',
        FunctionMetadataV10: 'FunctionMetadataV9',
        MetadataV10: {
            modules: 'Vec<ModuleMetadataV10>'
        },
        ModuleConstantMetadataV10: 'ModuleConstantMetadataV9',
        ModuleMetadataV10: {
            name: 'Text',
            storage: 'Option<StorageMetadataV10>',
            calls: 'Option<Vec<FunctionMetadataV10>>',
            events: 'Option<Vec<EventMetadataV10>>',
            constants: 'Vec<ModuleConstantMetadataV10>',
            errors: 'Vec<ErrorMetadataV10>'
        },
        StorageEntryModifierV10: 'StorageEntryModifierV9',
        StorageEntryMetadataV10: {
            name: 'Text',
            modifier: 'StorageEntryModifierV10',
            type: 'StorageEntryTypeV10',
            fallback: 'Bytes',
            docs: 'Vec<Text>'
        },
        StorageEntryTypeV10: {
            _enum: {
                Plain: 'Type',
                Map: {
                    hasher: 'StorageHasherV10',
                    key: 'Type',
                    value: 'Type',
                    linked: 'bool'
                },
                DoubleMap: {
                    hasher: 'StorageHasherV10',
                    key1: 'Type',
                    key2: 'Type',
                    value: 'Type',
                    key2Hasher: 'StorageHasherV10'
                }
            }
        },
        StorageMetadataV10: {
            prefix: 'Text',
            items: 'Vec<StorageEntryMetadataV10>'
        },
        StorageHasherV10: {
            _enum: {
                Blake2_128: null,
                Blake2_256: null,
                Blake2_128Concat: null,
                Twox128: null,
                Twox256: null,
                Twox64Concat: null
            }
        }
    };

    const v11 = {
        ErrorMetadataV11: 'ErrorMetadataV10',
        EventMetadataV11: 'EventMetadataV10',
        ExtrinsicMetadataV11: {
            version: 'u8',
            signedExtensions: 'Vec<Text>'
        },
        FunctionArgumentMetadataV11: 'FunctionArgumentMetadataV10',
        FunctionMetadataV11: 'FunctionMetadataV10',
        MetadataV11: {
            modules: 'Vec<ModuleMetadataV11>',
            extrinsic: 'ExtrinsicMetadataV11'
        },
        ModuleConstantMetadataV11: 'ModuleConstantMetadataV10',
        ModuleMetadataV11: {
            name: 'Text',
            storage: 'Option<StorageMetadataV11>',
            calls: 'Option<Vec<FunctionMetadataV11>>',
            events: 'Option<Vec<EventMetadataV11>>',
            constants: 'Vec<ModuleConstantMetadataV11>',
            errors: 'Vec<ErrorMetadataV11>'
        },
        StorageEntryModifierV11: 'StorageEntryModifierV10',
        StorageEntryMetadataV11: {
            name: 'Text',
            modifier: 'StorageEntryModifierV11',
            type: 'StorageEntryTypeV11',
            fallback: 'Bytes',
            docs: 'Vec<Text>'
        },
        StorageEntryTypeV11: {
            _enum: {
                Plain: 'Type',
                Map: {
                    hasher: 'StorageHasherV11',
                    key: 'Type',
                    value: 'Type',
                    linked: 'bool'
                },
                DoubleMap: {
                    hasher: 'StorageHasherV11',
                    key1: 'Type',
                    key2: 'Type',
                    value: 'Type',
                    key2Hasher: 'StorageHasherV11'
                }
            }
        },
        StorageMetadataV11: {
            prefix: 'Text',
            items: 'Vec<StorageEntryMetadataV11>'
        },
        StorageHasherV11: {
            _enum: AllHashers
        }
    };

    const v12 = {
        ErrorMetadataV12: 'ErrorMetadataV11',
        EventMetadataV12: 'EventMetadataV11',
        ExtrinsicMetadataV12: 'ExtrinsicMetadataV11',
        FunctionArgumentMetadataV12: 'FunctionArgumentMetadataV11',
        FunctionMetadataV12: 'FunctionMetadataV11',
        MetadataV12: {
            modules: 'Vec<ModuleMetadataV12>',
            extrinsic: 'ExtrinsicMetadataV12'
        },
        ModuleConstantMetadataV12: 'ModuleConstantMetadataV11',
        ModuleMetadataV12: {
            name: 'Text',
            storage: 'Option<StorageMetadataV12>',
            calls: 'Option<Vec<FunctionMetadataV12>>',
            events: 'Option<Vec<EventMetadataV12>>',
            constants: 'Vec<ModuleConstantMetadataV12>',
            errors: 'Vec<ErrorMetadataV12>',
            index: 'u8'
        },
        StorageEntryModifierV12: 'StorageEntryModifierV11',
        StorageEntryMetadataV12: 'StorageEntryMetadataV11',
        StorageEntryTypeV12: 'StorageEntryTypeV11',
        StorageMetadataV12: 'StorageMetadataV11',
        StorageHasherV12: 'StorageHasherV11'
    };

    const v13 = {
        ErrorMetadataV13: 'ErrorMetadataV12',
        EventMetadataV13: 'EventMetadataV12',
        ExtrinsicMetadataV13: 'ExtrinsicMetadataV12',
        FunctionArgumentMetadataV13: 'FunctionArgumentMetadataV12',
        FunctionMetadataV13: 'FunctionMetadataV12',
        MetadataV13: {
            modules: 'Vec<ModuleMetadataV13>',
            extrinsic: 'ExtrinsicMetadataV13'
        },
        ModuleConstantMetadataV13: 'ModuleConstantMetadataV12',
        ModuleMetadataV13: {
            name: 'Text',
            storage: 'Option<StorageMetadataV13>',
            calls: 'Option<Vec<FunctionMetadataV13>>',
            events: 'Option<Vec<EventMetadataV13>>',
            constants: 'Vec<ModuleConstantMetadataV13>',
            errors: 'Vec<ErrorMetadataV13>',
            index: 'u8'
        },
        StorageEntryModifierV13: 'StorageEntryModifierV12',
        StorageEntryMetadataV13: {
            name: 'Text',
            modifier: 'StorageEntryModifierV13',
            type: 'StorageEntryTypeV13',
            fallback: 'Bytes',
            docs: 'Vec<Text>'
        },
        StorageEntryTypeV13: {
            _enum: {
                Plain: 'Type',
                Map: {
                    hasher: 'StorageHasherV13',
                    key: 'Type',
                    value: 'Type',
                    linked: 'bool'
                },
                DoubleMap: {
                    hasher: 'StorageHasherV13',
                    key1: 'Type',
                    key2: 'Type',
                    value: 'Type',
                    key2Hasher: 'StorageHasherV13'
                },
                NMap: {
                    keyVec: 'Vec<Type>',
                    hashers: 'Vec<StorageHasherV13>',
                    value: 'Type'
                }
            }
        },
        StorageMetadataV13: {
            prefix: 'Text',
            items: 'Vec<StorageEntryMetadataV13>'
        },
        StorageHasherV13: 'StorageHasherV12'
    };

    const Si1Variant = {
        name: 'Text',
        fields: 'Vec<Si1Field>',
        index: 'u8',
        docs: 'Vec<Text>'
    };
    const v1$1 = {
        Si1Field: {
            name: 'Option<Text>',
            type: 'Si1LookupTypeId',
            typeName: 'Option<Text>',
            docs: 'Vec<Text>'
        },
        Si1LookupTypeId: 'Compact<u32>',
        Si1Path: 'Si0Path',
        Si1Type: {
            path: 'Si1Path',
            params: 'Vec<Si1TypeParameter>',
            def: 'Si1TypeDef',
            docs: 'Vec<Text>'
        },
        Si1TypeDef: {
            _enum: {
                Composite: 'Si1TypeDefComposite',
                Variant: 'Si1TypeDefVariant',
                Sequence: 'Si1TypeDefSequence',
                Array: 'Si1TypeDefArray',
                Tuple: 'Si1TypeDefTuple',
                Primitive: 'Si1TypeDefPrimitive',
                Compact: 'Si1TypeDefCompact',
                BitSequence: 'Si1TypeDefBitSequence',
                HistoricMetaCompat: 'Type'
            }
        },
        Si1TypeDefArray: {
            len: 'u32',
            type: 'Si1LookupTypeId'
        },
        Si1TypeDefBitSequence: {
            bitStoreType: 'Si1LookupTypeId',
            bitOrderType: 'Si1LookupTypeId'
        },
        Si1TypeDefCompact: {
            type: 'Si1LookupTypeId'
        },
        Si1TypeDefComposite: {
            fields: 'Vec<Si1Field>'
        },
        Si1TypeDefPrimitive: 'Si0TypeDefPrimitive',
        Si1TypeDefSequence: {
            type: 'Si1LookupTypeId'
        },
        Si1TypeDefTuple: 'Vec<Si1LookupTypeId>',
        Si1TypeParameter: {
            name: 'Text',
            type: 'Option<Si1LookupTypeId>'
        },
        Si1TypeDefVariant: {
            variants: 'Vec<Si1Variant>'
        },
        Si1Variant
    };

    const v14 = {
        PortableTypeV14: {
            id: 'Si1LookupTypeId',
            type: 'Si1Type'
        },
        ErrorMetadataV14: {
            ...Si1Variant,
            args: 'Vec<Type>'
        },
        EventMetadataV14: {
            ...Si1Variant,
            args: 'Vec<Type>'
        },
        FunctionArgumentMetadataV14: {
            name: 'Text',
            type: 'Type',
            typeName: 'Option<Type>'
        },
        FunctionMetadataV14: {
            ...Si1Variant,
            args: 'Vec<FunctionArgumentMetadataV14>'
        },
        ExtrinsicMetadataV14: {
            type: 'SiLookupTypeId',
            version: 'u8',
            signedExtensions: 'Vec<SignedExtensionMetadataV14>'
        },
        MetadataV14: {
            lookup: 'PortableRegistry',
            pallets: 'Vec<PalletMetadataV14>',
            extrinsic: 'ExtrinsicMetadataV14',
            type: 'SiLookupTypeId'
        },
        PalletCallMetadataV14: {
            type: 'SiLookupTypeId'
        },
        PalletConstantMetadataV14: {
            name: 'Text',
            type: 'SiLookupTypeId',
            value: 'Bytes',
            docs: 'Vec<Text>'
        },
        PalletErrorMetadataV14: {
            type: 'SiLookupTypeId'
        },
        PalletEventMetadataV14: {
            type: 'SiLookupTypeId'
        },
        PalletMetadataV14: {
            name: 'Text',
            storage: 'Option<PalletStorageMetadataV14>',
            calls: 'Option<PalletCallMetadataV14>',
            events: 'Option<PalletEventMetadataV14>',
            constants: 'Vec<PalletConstantMetadataV14>',
            errors: 'Option<PalletErrorMetadataV14>',
            index: 'u8'
        },
        PalletStorageMetadataV14: {
            prefix: 'Text',
            items: 'Vec<StorageEntryMetadataV14>'
        },
        SignedExtensionMetadataV14: {
            identifier: 'Text',
            type: 'SiLookupTypeId',
            additionalSigned: 'SiLookupTypeId'
        },
        StorageEntryMetadataV14: {
            name: 'Text',
            modifier: 'StorageEntryModifierV14',
            type: 'StorageEntryTypeV14',
            fallback: 'Bytes',
            docs: 'Vec<Text>'
        },
        StorageEntryModifierV14: 'StorageEntryModifierV13',
        StorageEntryTypeV14: {
            _enum: {
                Plain: 'SiLookupTypeId',
                Map: {
                    hashers: 'Vec<StorageHasherV14>',
                    key: 'SiLookupTypeId',
                    value: 'SiLookupTypeId'
                }
            }
        },
        StorageHasherV14: 'StorageHasherV13'
    };

    const v15 = {
        CustomMetadata15: {
            map: 'BTreeMap<Text, CustomValueMetadata15>'
        },
        CustomValueMetadata15: {
            type: 'SiLookupTypeId',
            value: 'Bytes'
        },
        ExtrinsicMetadataV15: {
            version: 'u8',
            addressType: 'SiLookupTypeId',
            callType: 'SiLookupTypeId',
            signatureType: 'SiLookupTypeId',
            extraType: 'SiLookupTypeId',
            signedExtensions: 'Vec<SignedExtensionMetadataV14>'
        },
        OuterEnums15: {
            callType: 'SiLookupTypeId',
            eventType: 'SiLookupTypeId',
            errorType: 'SiLookupTypeId'
        },
        PalletMetadataV15: {
            name: 'Text',
            storage: 'Option<PalletStorageMetadataV14>',
            calls: 'Option<PalletCallMetadataV14>',
            events: 'Option<PalletEventMetadataV14>',
            constants: 'Vec<PalletConstantMetadataV14>',
            errors: 'Option<PalletErrorMetadataV14>',
            index: 'u8',
            docs: 'Vec<Text>'
        },
        RuntimeApiMetadataV15: {
            name: 'Text',
            methods: 'Vec<RuntimeApiMethodMetadataV15>',
            docs: 'Vec<Text>'
        },
        RuntimeApiMethodMetadataV15: {
            name: 'Text',
            inputs: 'Vec<RuntimeApiMethodParamMetadataV15>',
            output: 'SiLookupTypeId',
            docs: 'Vec<Text>'
        },
        RuntimeApiMethodParamMetadataV15: {
            name: 'Text',
            type: 'SiLookupTypeId'
        },
        MetadataV15: {
            lookup: 'PortableRegistry',
            pallets: 'Vec<PalletMetadataV15>',
            extrinsic: 'ExtrinsicMetadataV15',
            type: 'SiLookupTypeId',
            apis: 'Vec<RuntimeApiMetadataV15>',
            outerEnums: 'OuterEnums15',
            custom: 'CustomMetadata15'
        }
    };

    const v16 = {
        MetadataV16: {
            lookup: 'PortableRegistry',
            pallets: 'Vec<PalletMetadataV16>',
            extrinsic: 'ExtrinsicMetadataV16',
            apis: 'Vec<RuntimeApiMetadataV16>',
            outerEnums: 'OuterEnums15',
            custom: 'CustomMetadata15'
        },
        PalletMetadataV16: {
            name: 'Text',
            storage: 'Option<PalletStorageMetadataV16>',
            calls: 'Option<PalletCallMetadataV16>',
            events: 'Option<PalletEventMetadataV16>',
            constants: 'Vec<PalletConstantMetadataV16>',
            errors: 'Option<PalletErrorMetadataV16>',
            associatedTypes: 'Vec<PalletAssociatedTypeMetadataV16>',
            viewFunctions: 'Vec<PalletViewFunctionMetadataV16>',
            index: 'u8',
            docs: 'Vec<Text>',
            deprecationInfo: 'ItemDeprecationInfoV16'
        },
        PalletStorageMetadataV16: {
            prefix: 'Text',
            items: 'Vec<StorageEntryMetadataV16>'
        },
        StorageEntryMetadataV16: {
            name: 'Text',
            modifier: 'StorageEntryModifierV14',
            type: 'StorageEntryTypeV14',
            fallback: 'Bytes',
            docs: 'Vec<Text>',
            deprecationInfo: 'ItemDeprecationInfoV16'
        },
        ItemDeprecationInfoV16: {
            _enum: {
                NotDeprecated: 'Null',
                DeprecatedWithoutNote: 'Null',
                Deprecated: {
                    note: 'Text',
                    since: 'Option<Text>'
                }
            }
        },
        PalletCallMetadataV16: {
            type: 'SiLookupTypeId',
            deprecationInfo: 'EnumDeprecationInfoV16'
        },
        EnumDeprecationInfoV16: 'BTreeMap<u8, VariantDeprecationInfoV16>',
        VariantDeprecationInfoV16: {
            _enum: {
                DummyVariant: 'Null',
                DeprecatedWithoutNote: 'Null',
                Deprecated: {
                    note: 'Text',
                    since: 'Option<Text>'
                }
            }
        },
        PalletEventMetadataV16: {
            type: 'SiLookupTypeId',
            deprecationInfo: 'EnumDeprecationInfoV16'
        },
        PalletConstantMetadataV16: {
            name: 'Text',
            type: 'SiLookupTypeId',
            value: 'Bytes',
            docs: 'Vec<Text>',
            deprecationInfo: 'ItemDeprecationInfoV16'
        },
        PalletErrorMetadataV16: {
            type: 'SiLookupTypeId',
            deprecationInfo: 'EnumDeprecationInfoV16'
        },
        PalletAssociatedTypeMetadataV16: {
            name: 'Text',
            type: 'SiLookupTypeId',
            docs: 'Vec<Text>'
        },
        PalletViewFunctionMetadataV16: {
            id: '[u8; 32]',
            name: 'Text',
            inputs: 'Vec<RuntimeApiMethodParamMetadataV15>',
            output: 'SiLookupTypeId',
            docs: 'Vec<T::String>',
            deprecationInfo: 'ItemDeprecationInfoV16'
        },
        ExtrinsicMetadataV16: {
            versions: 'Vec<u8>',
            addressType: 'SiLookupTypeId',
            callType: 'SiLookupTypeId',
            signatureType: 'SiLookupTypeId',
            transactionExtensionsByVersion: 'BTreeMap<u8, Vec<Compact<u32>>>',
            transactionExtensions: 'Vec<TransactionExtensionMetadataV16>'
        },
        TransactionExtensionMetadataV16: {
            identifier: 'Text',
            type: 'SiLookupTypeId',
            implicit: 'SiLookupTypeId'
        },
        RuntimeApiMetadataV16: {
            name: 'Text',
            methods: 'Vec<RuntimeApiMethodMetadataV16>',
            docs: 'Vec<Text>',
            version: 'Compact<u32>',
            deprecationInfo: 'ItemDeprecationInfoV16'
        },
        RuntimeApiMethodMetadataV16: {
            name: 'Text',
            inputs: 'Vec<RuntimeApiMethodParamMetadataV15>',
            output: 'SiLookupTypeId',
            docs: 'Vec<Text>',
            deprecationInfo: 'ItemDeprecationInfoV16'
        }
    };

    const definitions$1d = {
        rpc: {},
        runtime: runtime$z,
        types: {
            ...v9,
            ...v10,
            ...v11,
            ...v12,
            ...v13,
            ...v14,
            ...v15,
            ...v16,
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
            OpaqueMetadata: 'Opaque<Bytes>',
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

    const CORE_V1_TO_V4 = {
        execute_block: {
            description: 'Execute the given block.',
            params: [
                {
                    name: 'block',
                    type: 'Block'
                }
            ],
            type: 'Null'
        }
    };
    const CORE_V1_TO_V2 = {
        version: {
            description: 'Returns the version of the runtime.',
            params: [],
            type: 'RuntimeVersionPre3'
        }
    };
    const CORE_V2_TO_V4 = {
        initialize_block: {
            description: 'Initialize a block with the given header.',
            params: [
                {
                    name: 'header',
                    type: 'Header'
                }
            ],
            type: 'Null'
        }
    };
    const CORE_V4_VERSION = {
        version: {
            description: 'Returns the version of the runtime.',
            params: [],
            type: 'RuntimeVersion'
        }
    };
    const CORE_V4_TO_V5 = {
        ...CORE_V1_TO_V4,
        initialize_block: {
            description: 'Initialize a block with the given header.',
            params: [
                {
                    name: 'header',
                    type: 'Header'
                }
            ],
            type: 'ExtrinsicInclusionMode'
        }
    };
    const runtime$y = {
        Core: [
            {
                methods: {
                    ...CORE_V4_VERSION,
                    ...CORE_V4_TO_V5
                },
                version: 5
            },
            {
                methods: {
                    ...CORE_V4_VERSION,
                    ...CORE_V1_TO_V4,
                    ...CORE_V2_TO_V4
                },
                version: 4
            },
            {
                methods: {
                    version: {
                        description: 'Returns the version of the runtime.',
                        params: [],
                        type: 'RuntimeVersionPre4'
                    },
                    ...CORE_V1_TO_V4,
                    ...CORE_V2_TO_V4
                },
                version: 3
            },
            {
                methods: {
                    ...CORE_V1_TO_V2,
                    ...CORE_V1_TO_V4,
                    ...CORE_V2_TO_V4
                },
                version: 2
            },
            {
                methods: {
                    initialise_block: {
                        description: 'Initialize a block with the given header.',
                        params: [
                            {
                                name: 'header',
                                type: 'Header'
                            }
                        ],
                        type: 'Null'
                    },
                    ...CORE_V1_TO_V2,
                    ...CORE_V1_TO_V4
                },
                version: 1
            }
        ]
    };

    const numberTypes = {
        Fixed64: 'Int<64, Fixed64>',
        FixedI64: 'Int<64, FixedI64>',
        FixedU64: 'UInt<64, FixedU64>',
        Fixed128: 'Int<128, Fixed128>',
        FixedI128: 'Int<128, FixedI128>',
        FixedU128: 'UInt<128, FixedU128>',
        I32F32: 'Int<64, I32F32>',
        U32F32: 'UInt<64, U32F32>',
        PerU16: 'UInt<16, PerU16>',
        Perbill: 'UInt<32, Perbill>',
        Percent: 'UInt<8, Percent>',
        Permill: 'UInt<32, Permill>',
        Perquintill: 'UInt<64, Perquintill>'
    };
    const knownOrigins = {
        Council: 'CollectiveOrigin',
        System: 'SystemOrigin',
        TechnicalCommittee: 'CollectiveOrigin',
        Xcm: 'XcmOrigin',
        XcmPallet: 'XcmOrigin',
        Authority: 'AuthorityOrigin',
        GeneralCouncil: 'CollectiveOrigin'
    };
    const definitions$1c = {
        rpc: {},
        runtime: runtime$y,
        types: {
            ...numberTypes,
            AccountId: 'AccountId32',
            AccountId20: 'GenericEthereumAccountId',
            AccountId32: 'GenericAccountId32',
            AccountId33: 'GenericAccountId33',
            AccountIdOf: 'AccountId',
            AccountIndex: 'GenericAccountIndex',
            Address: 'MultiAddress',
            AssetId: 'u32',
            Balance: 'UInt<128, Balance>',
            BalanceOf: 'Balance',
            Block: 'GenericBlock',
            BlockNumber: 'u32',
            BlockNumberFor: 'BlockNumber',
            BlockNumberOf: 'BlockNumber',
            Call: 'GenericCall',
            CallHash: 'Hash',
            CallHashOf: 'CallHash',
            ChangesTrieConfiguration: {
                digestInterval: 'u32',
                digestLevels: 'u32'
            },
            ChangesTrieSignal: {
                _enum: {
                    NewConfiguration: 'Option<ChangesTrieConfiguration>'
                }
            },
            ConsensusEngineId: 'GenericConsensusEngineId',
            CodecHash: 'Hash',
            CrateVersion: {
                major: 'u16',
                minor: 'u8',
                patch: 'u8'
            },
            Digest: {
                logs: 'Vec<DigestItem>'
            },
            DigestItem: {
                _enum: {
                    Other: 'Bytes',
                    AuthoritiesChange: 'Vec<AuthorityId>',
                    ChangesTrieRoot: 'Hash',
                    SealV0: 'SealV0',
                    Consensus: 'Consensus',
                    Seal: 'Seal',
                    PreRuntime: 'PreRuntime',
                    ChangesTrieSignal: 'ChangesTrieSignal',
                    RuntimeEnvironmentUpdated: 'Null'
                }
            },
            ExtrinsicsWeight: {
                normal: 'Weight',
                operational: 'Weight'
            },
            H32: '[u8; 4; H32]',
            H64: '[u8; 8; H64]',
            H128: '[u8; 16; H128]',
            H160: '[u8; 20; H160]',
            H256: '[u8; 32; H256]',
            H512: '[u8; 64; H512]',
            H1024: '[u8; 128; H1024]',
            H2048: '[u8; 256; H2048]',
            Hash: 'H256',
            Header: {
                parentHash: 'Hash',
                number: 'Compact<BlockNumber>',
                stateRoot: 'Hash',
                extrinsicsRoot: 'Hash',
                digest: 'Digest'
            },
            HeaderPartial: {
                parentHash: 'Hash',
                number: 'BlockNumber'
            },
            IndicesLookupSource: 'GenericLookupSource',
            Index: 'u32',
            Justification: '(ConsensusEngineId, EncodedJustification)',
            EncodedJustification: 'Bytes',
            Justifications: 'Vec<Justification>',
            KeyValue: '(StorageKey, StorageData)',
            KeyTypeId: 'u32',
            LockIdentifier: '[u8; 8]',
            LookupSource: 'MultiAddress',
            LookupTarget: 'AccountId',
            ModuleId: 'LockIdentifier',
            MultiAddress: 'GenericMultiAddress',
            MultiSigner: {
                _enum: {
                    Ed25519: '[u8; 32]',
                    Sr25519: '[u8; 32]',
                    Ecdsa: '[u8; 33]'
                }
            },
            Moment: 'UInt<64, Moment>',
            OpaqueCall: 'Bytes',
            Origin: 'DoNotConstruct<Origin>',
            OriginCaller: {
                _enum: {
                    System: 'SystemOrigin'
                }
            },
            PalletId: 'LockIdentifier',
            PalletsOrigin: 'OriginCaller',
            PalletVersion: {
                major: 'u16',
                minor: 'u8',
                patch: 'u8'
            },
            Pays: {
                _enum: ['Yes', 'No']
            },
            Phantom: 'Null',
            PhantomData: 'Null',
            Releases: {
                _enum: ['V1', 'V2', 'V3', 'V4', 'V5', 'V6', 'V7', 'V8', 'V9', 'V10']
            },
            RuntimeCall: 'Call',
            RuntimeEvent: 'Event',
            RuntimeDbWeight: {
                read: 'Weight',
                write: 'Weight'
            },
            SignedBlock: 'SignedBlockWithJustifications',
            SignedBlockWithJustification: {
                block: 'Block',
                justification: 'Option<EncodedJustification>'
            },
            SignedBlockWithJustifications: {
                block: 'Block',
                justifications: 'Option<Justifications>'
            },
            Slot: 'u64',
            SlotDuration: 'u64',
            StorageData: 'Bytes',
            StorageInfo: {
                palletName: 'Bytes',
                storage_name: 'Bytes',
                prefix: 'Bytes',
                maxValues: 'Option<u32>',
                maxSize: 'Option<u32>'
            },
            StorageProof: {
                trieNodes: 'Vec<Bytes>'
            },
            TransactionPriority: 'u64',
            TransactionLongevity: 'u64',
            TransactionTag: 'Bytes',
            TransactionInfo: {
                _alias: {
                    dataSize: 'size'
                },
                chunkRoot: 'H256',
                contentHash: 'H256',
                dataSize: 'u32',
                blockChunks: 'u32'
            },
            TransactionStorageProof: {
                chunk: 'Vec<u8>',
                proof: 'Vec<Vec<u8>>'
            },
            ValidatorId: 'AccountId',
            ValidatorIdOf: 'ValidatorId',
            WeightV0: 'u32',
            WeightV1: 'u64',
            WeightV2: {
                refTime: 'Compact<u64>',
                proofSize: 'Compact<u64>'
            },
            Weight: 'WeightV2',
            WeightMultiplier: 'Fixed64',
            PreRuntime: '(ConsensusEngineId, Bytes)',
            SealV0: '(u64, Signature)',
            Seal: '(ConsensusEngineId, Bytes)',
            Consensus: '(ConsensusEngineId, Bytes)',
            ExtrinsicInclusionMode: {
                _enum: ['AllExtrinsics', 'OnlyInherents']
            }
        }
    };

    const v0$1 = {
        Si0Field: {
            name: 'Option<Text>',
            type: 'Si0LookupTypeId',
            typeName: 'Option<Text>',
            docs: 'Vec<Text>'
        },
        Si0LookupTypeId: 'u32',
        Si0Path: 'Vec<Text>',
        Si0Type: {
            path: 'Si0Path',
            params: 'Vec<Si0LookupTypeId>',
            def: 'Si0TypeDef'
        },
        Si0TypeDef: {
            _enum: {
                Composite: 'Si0TypeDefComposite',
                Variant: 'Si0TypeDefVariant',
                Sequence: 'Si0TypeDefSequence',
                Array: 'Si0TypeDefArray',
                Tuple: 'Si0TypeDefTuple',
                Primitive: 'Si0TypeDefPrimitive',
                Compact: 'Si0TypeDefCompact',
                Phantom: 'Si0TypeDefPhantom',
                BitSequence: 'Si0TypeDefBitSequence'
            }
        },
        Si0TypeDefArray: {
            len: 'u32',
            type: 'Si0LookupTypeId'
        },
        Si0TypeDefBitSequence: {
            bitStoreType: 'Si0LookupTypeId',
            bitOrderType: 'Si0LookupTypeId'
        },
        Si0TypeDefCompact: {
            type: 'Si0LookupTypeId'
        },
        Si0TypeDefComposite: {
            fields: 'Vec<Si0Field>'
        },
        Si0TypeDefPhantom: 'Null',
        Si0TypeDefVariant: {
            variants: 'Vec<Si0Variant>'
        },
        Si0TypeDefPrimitive: {
            _enum: ['Bool', 'Char', 'Str', 'U8', 'U16', 'U32', 'U64', 'U128', 'U256', 'I8', 'I16', 'I32', 'I64', 'I128', 'I256']
        },
        Si0TypeDefSequence: {
            type: 'Si0LookupTypeId'
        },
        Si0TypeDefTuple: 'Vec<Si0LookupTypeId>',
        Si0TypeParameter: {
            name: 'Text',
            type: 'Option<Si0LookupTypeId>'
        },
        Si0Variant: {
            name: 'Text',
            fields: 'Vec<Si0Field>',
            index: 'Option<u8>',
            discriminant: 'Option<u64>',
            docs: 'Vec<Text>'
        }
    };

    const definitions$1b = {
        rpc: {},
        types: {
            ...v0$1,
            ...v1$1,
            SiField: 'Si1Field',
            SiLookupTypeId: 'Si1LookupTypeId',
            SiPath: 'Si1Path',
            SiType: 'Si1Type',
            SiTypeDef: 'Si1TypeDef',
            SiTypeDefArray: 'Si1TypeDefArray',
            SiTypeDefBitSequence: 'Si1TypeDefBitSequence',
            SiTypeDefCompact: 'Si1TypeDefCompact',
            SiTypeDefComposite: 'Si1TypeDefComposite',
            SiTypeDefPrimitive: 'Si1TypeDefPrimitive',
            SiTypeDefSequence: 'Si1TypeDefSequence',
            SiTypeDefTuple: 'Si1TypeDefTuple',
            SiTypeParameter: 'Si1TypeParameter',
            SiTypeDefVariant: 'Si1TypeDefVariant',
            SiVariant: 'Si1Variant'
        }
    };

    const runtime$x = {
        AssetConversionApi: [
            {
                methods: {
                    get_reserves: {
                        description: 'Get pool reserves',
                        params: [
                            {
                                name: 'asset1',
                                type: 'StagingXcmV3MultiLocation'
                            },
                            {
                                name: 'asset2',
                                type: 'StagingXcmV3MultiLocation'
                            }
                        ],
                        type: 'Option<(Balance,Balance)>'
                    },
                    quote_price_exact_tokens_for_tokens: {
                        description: 'Quote price: exact tokens for tokens',
                        params: [
                            {
                                name: 'asset1',
                                type: 'StagingXcmV3MultiLocation'
                            },
                            {
                                name: 'asset2',
                                type: 'StagingXcmV3MultiLocation'
                            },
                            {
                                name: 'amount',
                                type: 'u128'
                            },
                            {
                                name: 'include_fee',
                                type: 'bool'
                            }
                        ],
                        type: 'Option<(Balance)>'
                    },
                    quote_price_tokens_for_exact_tokens: {
                        description: 'Quote price: tokens for exact tokens',
                        params: [
                            {
                                name: 'asset1',
                                type: 'StagingXcmV3MultiLocation'
                            },
                            {
                                name: 'asset2',
                                type: 'StagingXcmV3MultiLocation'
                            },
                            {
                                name: 'amount',
                                type: 'u128'
                            },
                            {
                                name: 'include_fee',
                                type: 'bool'
                            }
                        ],
                        type: 'Option<(Balance)>'
                    }
                },
                version: 1
            }
        ]
    };

    const definitions$1a = {
        rpc: {},
        runtime: runtime$x,
        types: {
            TAssetConversion: 'Option<MultiLocation>'
        }
    };

    const runtime$w = {
        AssetsApi: [
            {
                methods: {
                    account_balances: {
                        description: 'Return the current set of authorities.',
                        params: [
                            {
                                name: 'account',
                                type: 'AccountId'
                            }
                        ],
                        type: 'Vec<(u32, TAssetBalance)>'
                    }
                },
                version: 1
            }
        ]
    };

    const definitions$19 = {
        rpc: {},
        runtime: runtime$w,
        types: {
            AssetApprovalKey: {
                owner: 'AccountId',
                delegate: 'AccountId'
            },
            AssetApproval: {
                amount: 'TAssetBalance',
                deposit: 'TAssetDepositBalance'
            },
            AssetBalance: {
                balance: 'TAssetBalance',
                isFrozen: 'bool',
                isSufficient: 'bool'
            },
            AssetDestroyWitness: {
                accounts: 'Compact<u32>',
                sufficients: 'Compact<u32>',
                approvals: 'Compact<u32>'
            },
            AssetDetails: {
                owner: 'AccountId',
                issuer: 'AccountId',
                admin: 'AccountId',
                freezer: 'AccountId',
                supply: 'TAssetBalance',
                deposit: 'TAssetDepositBalance',
                minBalance: 'TAssetBalance',
                isSufficient: 'bool',
                accounts: 'u32',
                sufficients: 'u32',
                approvals: 'u32',
                isFrozen: 'bool'
            },
            AssetMetadata: {
                deposit: 'TAssetDepositBalance',
                name: 'Vec<u8>',
                symbol: 'Vec<u8>',
                decimals: 'u8',
                isFrozen: 'bool'
            },
            TAssetBalance: 'u64',
            TAssetDepositBalance: 'BalanceOf'
        }
    };

    const runtime$v = {
        AuraApi: [
            {
                methods: {
                    authorities: {
                        description: 'Return the current set of authorities.',
                        params: [],
                        type: 'Vec<AuthorityId>'
                    },
                    slot_duration: {
                        description: 'Returns the slot duration for Aura.',
                        params: [],
                        type: 'SlotDuration'
                    }
                },
                version: 1
            }
        ]
    };

    const definitions$18 = {
        rpc: {},
        runtime: runtime$v,
        types: {
            RawAuraPreDigest: {
                slotNumber: 'u64'
            }
        }
    };

    const definitions$17 = {
        rpc: {},
        types: {
            UncleEntryItem: {
                _enum: {
                    InclusionHeight: 'BlockNumber',
                    Uncle: '(Hash, Option<AccountId>)'
                }
            }
        }
    };

    const rpc$g = {
        epochAuthorship: {
            description: 'Returns data about which slots (primary or secondary) can be claimed in the current epoch with the keys in the keystore',
            isUnsafe: true,
            params: [],
            type: 'HashMap<AuthorityId, EpochAuthorship>'
        }
    };

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
    const runtime$u = {
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

    const definitions$16 = {
        rpc: rpc$g,
        runtime: runtime$u,
        types: {
            AllowedSlots: {
                _enum: ['PrimarySlots', 'PrimaryAndSecondaryPlainSlots', 'PrimaryAndSecondaryVRFSlots']
            },
            BabeAuthorityWeight: 'u64',
            BabeEpochConfiguration: {
                c: '(u64, u64)',
                allowedSlots: 'AllowedSlots'
            },
            BabeBlockWeight: 'u32',
            BabeEquivocationProof: {
                offender: 'AuthorityId',
                slotNumber: 'SlotNumber',
                firstHeader: 'Header',
                secondHeader: 'Header'
            },
            BabeGenesisConfiguration: {
                slotDuration: 'u64',
                epochLength: 'u64',
                c: '(u64, u64)',
                genesisAuthorities: 'Vec<(AuthorityId, BabeAuthorityWeight)>',
                randomness: 'Randomness',
                allowedSlots: 'AllowedSlots'
            },
            BabeGenesisConfigurationV1: {
                slotDuration: 'u64',
                epochLength: 'u64',
                c: '(u64, u64)',
                genesisAuthorities: 'Vec<(AuthorityId, BabeAuthorityWeight)>',
                randomness: 'Randomness',
                secondarySlots: 'bool'
            },
            BabeWeight: 'u64',
            MaybeRandomness: 'Option<Randomness>',
            MaybeVrf: 'Option<VrfData>',
            Epoch: {
                epochIndex: 'u64',
                startSlot: 'Slot',
                duration: 'u64',
                authorities: 'Vec<(AuthorityId, BabeAuthorityWeight)>',
                randomness: 'Hash',
                config: 'BabeEpochConfiguration'
            },
            EpochAuthorship: {
                primary: 'Vec<u64>',
                secondary: 'Vec<u64>',
                secondary_vrf: 'Vec<u64>'
            },
            NextConfigDescriptor: {
                _enum: {
                    V0: 'Null',
                    V1: 'NextConfigDescriptorV1'
                }
            },
            NextConfigDescriptorV1: {
                c: '(u64, u64)',
                allowedSlots: 'AllowedSlots'
            },
            OpaqueKeyOwnershipProof: 'Bytes',
            Randomness: 'Hash',
            RawBabePreDigest: {
                _enum: {
                    Phantom: 'Null',
                    Primary: 'RawBabePreDigestPrimary',
                    SecondaryPlain: 'RawBabePreDigestSecondaryPlain',
                    SecondaryVRF: 'RawBabePreDigestSecondaryVRF'
                }
            },
            RawBabePreDigestPrimary: {
                authorityIndex: 'u32',
                slotNumber: 'SlotNumber',
                vrfOutput: 'VrfOutput',
                vrfProof: 'VrfProof'
            },
            RawBabePreDigestSecondaryPlain: {
                authorityIndex: 'u32',
                slotNumber: 'SlotNumber'
            },
            RawBabePreDigestSecondaryVRF: {
                authorityIndex: 'u32',
                slotNumber: 'SlotNumber',
                vrfOutput: 'VrfOutput',
                vrfProof: 'VrfProof'
            },
            RawBabePreDigestTo159: {
                _enum: {
                    Primary: 'RawBabePreDigestPrimaryTo159',
                    Secondary: 'RawBabePreDigestSecondaryTo159'
                }
            },
            RawBabePreDigestPrimaryTo159: {
                authorityIndex: 'u32',
                slotNumber: 'SlotNumber',
                weight: 'BabeBlockWeight',
                vrfOutput: 'VrfOutput',
                vrfProof: 'VrfProof'
            },
            RawBabePreDigestSecondaryTo159: {
                authorityIndex: 'u32',
                slotNumber: 'SlotNumber',
                weight: 'BabeBlockWeight'
            },
            RawBabePreDigestCompat: {
                _enum: {
                    Zero: 'u32',
                    One: 'u32',
                    Two: 'u32',
                    Three: 'u32'
                }
            },
            SlotNumber: 'u64',
            VrfData: '[u8; 32]',
            VrfOutput: '[u8; 32]',
            VrfProof: '[u8; 64]'
        }
    };

    const definitions$15 = {
        rpc: {},
        types: {
            AccountData: {
                free: 'Balance',
                reserved: 'Balance',
                miscFrozen: 'Balance',
                feeFrozen: 'Balance'
            },
            BalanceLockTo212: {
                id: 'LockIdentifier',
                amount: 'Balance',
                until: 'BlockNumber',
                reasons: 'WithdrawReasons'
            },
            BalanceLock: {
                id: 'LockIdentifier',
                amount: 'Balance',
                reasons: 'Reasons'
            },
            BalanceStatus: {
                _enum: ['Free', 'Reserved']
            },
            Reasons: {
                _enum: ['Fee', 'Misc', 'All']
            },
            ReserveData: {
                id: 'ReserveIdentifier',
                amount: 'Balance'
            },
            ReserveIdentifier: '[u8; 8]',
            VestingSchedule: {
                offset: 'Balance',
                perBlock: 'Balance',
                startingBlock: 'BlockNumber'
            },
            WithdrawReasons: {
                _set: {
                    TransactionPayment: 0b0000_0001,
                    Transfer: 0b0000_0010,
                    Reserve: 0b0000_0100,
                    Fee: 0b0000_1000,
                    Tip: 0b0001_0000
                }
            }
        }
    };

    const rpc$f = {
        getFinalizedHead: {
            description: 'Returns hash of the latest BEEFY finalized block as seen by this client.',
            params: [],
            type: 'H256'
        },
        subscribeJustifications: {
            description: 'Returns the block most recently finalized by BEEFY, alongside its justification.',
            params: [],
            pubsub: [
                'justifications',
                'subscribeJustifications',
                'unsubscribeJustifications'
            ],
            type: 'BeefyVersionedFinalityProof'
        }
    };

    const BEEFY_V3 = {
        beefy_genesis: {
            description: 'Return the block number where BEEFY consensus is enabled/started',
            params: [],
            type: 'Option<BlockNumber>'
        },
        generate_key_ownership_proof: {
            description: 'Generates a proof of key ownership for the given authority in the given set.',
            params: [
                {
                    name: 'setId',
                    type: 'ValidatorSetId'
                },
                {
                    name: 'authorityId',
                    type: 'AuthorityId'
                }
            ],
            type: 'Option<OpaqueKeyOwnershipProof>'
        },
        validator_set: {
            description: 'Return the current active BEEFY validator set',
            params: [],
            type: 'Option<ValidatorSet>'
        }
    };
    const BEEFY_V1_V3 = {
        ...BEEFY_V3,
        submit_report_equivocation_unsigned_extrinsic: {
            description: 'Submits an unsigned extrinsic to report an equivocation.',
            params: [
                {
                    name: 'equivocationProof',
                    type: 'BeefyEquivocationProof'
                },
                {
                    name: 'keyOwnerProof',
                    type: 'OpaqueKeyOwnershipProof'
                }
            ],
            type: 'Option<Null>'
        }
    };
    const BEEFY_V4 = {
        ...BEEFY_V3,
        submit_report_double_voting_unsigned_extrinsic: {
            description: 'Submits an unsigned extrinsic to report a double voting equivocation.',
            params: [
                {
                    name: 'equivocationProof',
                    type: 'SpConsensusBeefyDoubleVotingProof'
                },
                {
                    name: 'keyOwnerProof',
                    type: 'OpaqueKeyOwnershipProof'
                }
            ],
            type: 'Option<Null>'
        }
    };
    const BEEFY_MMR_V1 = {
        authority_set_proof: {
            description: 'Return the currently active BEEFY authority set proof.',
            params: [],
            type: 'BeefyAuthoritySet'
        },
        next_authority_set_proof: {
            description: 'Return the next/queued BEEFY authority set proof.',
            params: [],
            type: 'BeefyNextAuthoritySet'
        }
    };
    const runtime$t = {
        BeefyApi: [
            {
                methods: BEEFY_V4,
                version: 4
            },
            {
                methods: BEEFY_V1_V3,
                version: 3
            },
            {
                methods: BEEFY_V1_V3,
                version: 2
            },
            {
                methods: BEEFY_V1_V3,
                version: 1
            }
        ],
        BeefyMmrApi: [
            {
                methods: BEEFY_MMR_V1,
                version: 1
            }
        ]
    };

    const definitions$14 = {
        rpc: rpc$f,
        runtime: runtime$t,
        types: {
            BeefyAuthoritySet: {
                id: 'u64',
                len: 'u32',
                root: 'H256'
            },
            BeefyCommitment: {
                payload: 'BeefyPayload',
                blockNumber: 'BlockNumber',
                validatorSetId: 'ValidatorSetId'
            },
            BeefyId: '[u8; 33]',
            BeefyEquivocationProof: {
                first: 'BeefyVoteMessage',
                second: 'BeefyVoteMessage'
            },
            BeefyCompactSignedCommitment: {
                commitment: 'BeefyCommitment',
                signaturesFrom: 'Vec<u8>',
                validatorSetLen: 'u32',
                signaturesCompact: 'Vec<EcdsaSignature>'
            },
            BeefySignedCommitment: {
                commitment: 'BeefyCommitment',
                signatures: 'Vec<Option<EcdsaSignature>>'
            },
            BeefyVersionedFinalityProof: {
                _enum: {
                    V0: 'Null',
                    V1: 'BeefyCompactSignedCommitment'
                }
            },
            BeefyNextAuthoritySet: {
                id: 'u64',
                len: 'u32',
                root: 'H256'
            },
            BeefyPayload: 'Vec<(BeefyPayloadId, Bytes)>',
            BeefyPayloadId: '[u8;2]',
            BeefyVoteMessage: {
                commitment: 'BeefyCommitment',
                id: 'AuthorityId',
                signature: 'Signature'
            },
            MmrRootHash: 'H256',
            ValidatorSetId: 'u64',
            ValidatorSet: {
                validators: 'Vec<AuthorityId>',
                id: 'ValidatorSetId'
            }
        }
    };

    const runtime$s = {
        Benchmark: [
            {
                methods: {
                    benchmark_metadata: {
                        description: 'Get the benchmark metadata available for this runtime.',
                        params: [
                            {
                                name: 'extra',
                                type: 'bool'
                            }
                        ],
                        type: '(Vec<BenchmarkList>, Vec<StorageInfo>)'
                    },
                    dispatch_benchmark: {
                        description: 'Dispatch the given benchmark.',
                        params: [
                            {
                                name: 'config',
                                type: 'BenchmarkConfig'
                            }
                        ],
                        type: 'Result<Vec<BenchmarkBatch>, Text>'
                    }
                },
                version: 1
            }
        ]
    };

    const definitions$13 = {
        rpc: {},
        runtime: runtime$s,
        types: {
            BenchmarkBatch: {
                pallet: 'Text',
                instance: 'Text',
                benchmark: 'Text',
                results: 'Vec<BenchmarkResult>'
            },
            BenchmarkConfig: {
                pallet: 'Bytes',
                benchmark: 'Bytes',
                selectedComponents: 'Vec<(BenchmarkParameter, u32)>',
                verify: 'bool',
                internalRepeats: 'u32'
            },
            BenchmarkList: {
                pallet: 'Bytes',
                instance: 'Bytes',
                benchmarks: 'Vec<BenchmarkMetadata>'
            },
            BenchmarkMetadata: {
                name: 'Bytes',
                components: 'Vec<(BenchmarkParameter, u32, u32)>'
            },
            BenchmarkParameter: {
                _enum: ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z']
            },
            BenchmarkResult: {
                components: 'Vec<(BenchmarkParameter, u32)>',
                extrinsicTime: 'u128',
                storageRootTime: 'u128',
                reads: 'u32',
                repeatReads: 'u32',
                writes: 'u32',
                repeatWrites: 'u32',
                proofSize: 'u32',
                benchKeys: 'Vec<(Vec<u8>, u32, u32, bool)>'
            }
        }
    };

    const BB_V2_TO_V4 = {
        random_seed: {
            description: 'Generate a random seed.',
            params: [],
            type: 'Hash'
        }
    };
    const BB_V2_TO_V5 = {
        apply_extrinsic: {
            description: 'Apply the given extrinsic.',
            params: [
                {
                    name: 'extrinsic',
                    type: 'Extrinsic'
                }
            ],
            type: 'ApplyExtrinsicResultPre6'
        }
    };
    const BB_V2_TO_V6 = {
        check_inherents: {
            description: 'Check that the inherents are valid.',
            params: [
                {
                    name: 'block',
                    type: 'Block'
                },
                {
                    name: 'data',
                    type: 'InherentData'
                }
            ],
            type: 'CheckInherentsResult'
        },
        inherent_extrinsics: {
            description: 'Generate inherent extrinsics.',
            params: [
                {
                    name: 'inherent',
                    type: 'InherentData'
                }
            ],
            type: 'Vec<Extrinsic>'
        }
    };
    const BB_V3_TO_V6 = {
        finalize_block: {
            description: 'Finish the current block.',
            params: [],
            type: 'Header'
        }
    };
    const runtime$r = {
        BlockBuilder: [
            {
                methods: {
                    apply_extrinsic: {
                        description: 'Apply the given extrinsic.',
                        params: [
                            {
                                name: 'extrinsic',
                                type: 'Extrinsic'
                            }
                        ],
                        type: 'ApplyExtrinsicResult'
                    },
                    ...BB_V2_TO_V6,
                    ...BB_V3_TO_V6
                },
                version: 6
            },
            {
                methods: {
                    ...BB_V2_TO_V5,
                    ...BB_V2_TO_V6,
                    ...BB_V3_TO_V6
                },
                version: 5
            },
            {
                methods: {
                    ...BB_V2_TO_V4,
                    ...BB_V2_TO_V5,
                    ...BB_V2_TO_V6,
                    ...BB_V3_TO_V6
                },
                version: 4
            },
            {
                methods: {
                    ...BB_V2_TO_V4,
                    ...BB_V2_TO_V6,
                    ...BB_V3_TO_V6
                },
                version: 3
            },
            {
                methods: {
                    finalise_block: {
                        description: 'Finish the current block.',
                        params: [],
                        type: 'Header'
                    },
                    ...BB_V2_TO_V4,
                    ...BB_V2_TO_V6
                },
                version: 2
            }
        ]
    };

    const definitions$12 = {
        rpc: {},
        runtime: runtime$r,
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

    const definitions$11 = {
        rpc: {},
        types: {
            CollectiveOrigin: {
                _enum: {
                    Members: '(MemberCount, MemberCount)',
                    Member: 'AccountId'
                }
            },
            MemberCount: 'u32',
            ProposalIndex: 'u32',
            VotesTo230: {
                index: 'ProposalIndex',
                threshold: 'MemberCount',
                ayes: 'Vec<AccountId>',
                nays: 'Vec<AccountId>'
            },
            Votes: {
                index: 'ProposalIndex',
                threshold: 'MemberCount',
                ayes: 'Vec<AccountId>',
                nays: 'Vec<AccountId>',
                end: 'BlockNumber'
            }
        }
    };

    const definitions$10 = {
        rpc: {},
        types: {
            AuthorityId: 'AccountId',
            RawVRFOutput: '[u8; 32]'
        }
    };

    const rpc$e = {
        call: {
            deprecated: 'Use the runtime interface `api.call.contractsApi.call` instead',
            description: 'Executes a call to a contract',
            params: [
                {
                    name: 'callRequest',
                    type: 'ContractCallRequest'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'ContractExecResult'
        },
        getStorage: {
            deprecated: 'Use the runtime interface `api.call.contractsApi.getStorage` instead',
            description: 'Returns the value under a specified storage key in a contract',
            params: [
                {
                    name: 'address',
                    type: 'AccountId'
                },
                {
                    name: 'key',
                    type: 'H256'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'Option<Bytes>'
        },
        instantiate: {
            deprecated: 'Use the runtime interface `api.call.contractsApi.instantiate` instead',
            description: 'Instantiate a new contract',
            params: [
                {
                    name: 'request',
                    type: 'InstantiateRequestV1'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'ContractInstantiateResult'
        },
        rentProjection: {
            deprecated: 'Not available in newer versions of the contracts interfaces',
            description: 'Returns the projected time a given contract will be able to sustain paying its rent',
            params: [
                {
                    name: 'address',
                    type: 'AccountId'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'Option<BlockNumber>'
        },
        uploadCode: {
            deprecated: 'Use the runtime interface `api.call.contractsApi.uploadCode` instead',
            description: 'Upload new code without instantiating a contract from it',
            endpoint: 'contracts_upload_code',
            params: [
                {
                    name: 'uploadRequest',
                    type: 'CodeUploadRequest'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'CodeUploadResult'
        }
    };

    const SHARED_V1_V2 = {
        get_storage: {
            description: 'Query a given storage key in a given contract.',
            params: [
                {
                    name: 'address',
                    type: 'AccountId'
                },
                {
                    name: 'key',
                    type: 'Bytes'
                }
            ],
            type: 'Option<Bytes>'
        },
        upload_code: {
            description: 'Upload new code without instantiating a contract from it.',
            params: [
                {
                    name: 'origin',
                    type: 'AccountId'
                },
                {
                    name: 'code',
                    type: 'Bytes'
                },
                {
                    name: 'storageDepositLimit',
                    type: 'Option<Balance>'
                }
            ],
            type: 'CodeUploadResult'
        }
    };
    const runtime$q = {
        ContractsApi: [
            {
                methods: {
                    call: {
                        description: 'Perform a call from a specified account to a given contract.',
                        params: [
                            {
                                name: 'origin',
                                type: 'AccountId'
                            },
                            {
                                name: 'dest',
                                type: 'AccountId'
                            },
                            {
                                name: 'value',
                                type: 'Balance'
                            },
                            {
                                name: 'gasLimit',
                                type: 'Option<WeightV2>'
                            },
                            {
                                name: 'storageDepositLimit',
                                type: 'Option<Balance>'
                            },
                            {
                                name: 'inputData',
                                type: 'Vec<u8>'
                            }
                        ],
                        type: 'ContractExecResult'
                    },
                    instantiate: {
                        description: 'Instantiate a new contract.',
                        params: [
                            {
                                name: 'origin',
                                type: 'AccountId'
                            },
                            {
                                name: 'value',
                                type: 'Balance'
                            },
                            {
                                name: 'gasLimit',
                                type: 'Option<WeightV2>'
                            },
                            {
                                name: 'storageDepositLimit',
                                type: 'Option<Balance>'
                            },
                            {
                                name: 'code',
                                type: 'CodeSource'
                            },
                            {
                                name: 'data',
                                type: 'Bytes'
                            },
                            {
                                name: 'salt',
                                type: 'Bytes'
                            }
                        ],
                        type: 'ContractInstantiateResult'
                    },
                    ...SHARED_V1_V2
                },
                version: 2
            },
            {
                methods: {
                    call: {
                        description: 'Perform a call from a specified account to a given contract.',
                        params: [
                            {
                                name: 'origin',
                                type: 'AccountId'
                            },
                            {
                                name: 'dest',
                                type: 'AccountId'
                            },
                            {
                                name: 'value',
                                type: 'Balance'
                            },
                            {
                                name: 'gasLimit',
                                type: 'u64'
                            },
                            {
                                name: 'storageDepositLimit',
                                type: 'Option<Balance>'
                            },
                            {
                                name: 'inputData',
                                type: 'Vec<u8>'
                            }
                        ],
                        type: 'ContractExecResultU64'
                    },
                    instantiate: {
                        description: 'Instantiate a new contract.',
                        params: [
                            {
                                name: 'origin',
                                type: 'AccountId'
                            },
                            {
                                name: 'value',
                                type: 'Balance'
                            },
                            {
                                name: 'gasLimit',
                                type: 'u64'
                            },
                            {
                                name: 'storageDepositLimit',
                                type: 'Option<Balance>'
                            },
                            {
                                name: 'code',
                                type: 'CodeSource'
                            },
                            {
                                name: 'data',
                                type: 'Bytes'
                            },
                            {
                                name: 'salt',
                                type: 'Bytes'
                            }
                        ],
                        type: 'ContractInstantiateResultU64'
                    },
                    ...SHARED_V1_V2
                },
                version: 1
            }
        ]
    };

    const definitions$$ = {
        rpc: rpc$e,
        runtime: runtime$q,
        types: {
            AliveContractInfo: {
                trieId: 'TrieId',
                storageSize: 'u32',
                pairCount: 'u32',
                codeHash: 'CodeHash',
                rentAllowance: 'Balance',
                rentPaid: 'Balance',
                deductBlock: 'BlockNumber',
                lastWrite: 'Option<BlockNumber>',
                _reserved: 'Option<Null>'
            },
            CodeHash: 'Hash',
            CodeSource: {
                _enum: {
                    Upload: 'Bytes',
                    Existing: 'Hash'
                }
            },
            CodeUploadRequest: {
                origin: 'AccountId',
                code: 'Bytes',
                storageDepositLimit: 'Option<Balance>'
            },
            CodeUploadResult: 'Result<CodeUploadResultValue, DispatchError>',
            CodeUploadResultValue: {
                codeHash: 'CodeHash',
                deposit: 'Balance'
            },
            ContractCallRequest: {
                origin: 'AccountId',
                dest: 'AccountId',
                value: 'Balance',
                gasLimit: 'u64',
                storageDepositLimit: 'Option<Balance>',
                inputData: 'Bytes'
            },
            ContractExecResultSuccessTo255: {
                status: 'u8',
                data: 'Raw'
            },
            ContractExecResultTo255: {
                _enum: {
                    Success: 'ContractExecResultSuccessTo255',
                    Error: 'Null'
                }
            },
            ContractExecResultSuccessTo260: {
                flags: 'ContractReturnFlags',
                data: 'Bytes',
                gasConsumed: 'u64'
            },
            ContractExecResultTo260: {
                _enum: {
                    Success: 'ContractExecResultSuccessTo260',
                    Error: 'Null'
                }
            },
            ContractExecResultOk: {
                flags: 'ContractReturnFlags',
                data: 'Bytes'
            },
            ContractExecResultResult: 'Result<ContractExecResultOk, DispatchError>',
            ContractExecResultTo267: {
                gasConsumed: 'u64',
                debugMessage: 'Text',
                result: 'ContractExecResultResult'
            },
            ContractExecResult: {
                gasConsumed: 'Weight',
                gasRequired: 'Weight',
                storageDeposit: 'StorageDeposit',
                debugMessage: 'Text',
                result: 'ContractExecResultResult'
            },
            ContractExecResultU64: {
                gasConsumed: 'u64',
                gasRequired: 'u64',
                storageDeposit: 'StorageDeposit',
                debugMessage: 'Text',
                result: 'ContractExecResultResult'
            },
            ContractInfo: {
                _enum: {
                    Alive: 'AliveContractInfo',
                    Tombstone: 'TombstoneContractInfo'
                }
            },
            ContractCallFlags: {
                _set: {
                    _bitLength: 32,
                    ForwardInput: 0b0000_0001,
                    CloneInput: 0b0000_0010,
                    TailCall: 0b0000_0100,
                    AllowReentry: 0b0000_1000
                }
            },
            ContractReturnFlags: {
                _set: {
                    _bitLength: 32,
                    Revert: 0x0000_0001
                }
            },
            ContractStorageKey: '[u8; 32]',
            DeletedContract: {
                pairCount: 'u32',
                trieId: 'TrieId'
            },
            ExecReturnValue: {
                flags: 'ContractReturnFlags',
                data: 'Bytes'
            },
            Gas: 'u64',
            HostFnWeightsTo264: {
                caller: 'Weight',
                address: 'Weight',
                gasLeft: 'Weight',
                balance: 'Weight',
                valueTransferred: 'Weight',
                minimumBalance: 'Weight',
                tombstoneDeposit: 'Weight',
                rentAllowance: 'Weight',
                blockNumber: 'Weight',
                now: 'Weight',
                weightToFee: 'Weight',
                gas: 'Weight',
                input: 'Weight',
                inputPerByte: 'Weight',
                return: 'Weight',
                returnPerByte: 'Weight',
                terminate: 'Weight',
                restoreTo: 'Weight',
                restoreToPerDelta: 'Weight',
                random: 'Weight',
                depositEvent: 'Weight',
                depositEventPerTopic: 'Weight',
                depositEventPerByte: 'Weight',
                setRentAllowance: 'Weight',
                setStorage: 'Weight',
                setStoragePerByte: 'Weight',
                clearStorage: 'Weight',
                getStorage: 'Weight',
                getStoragePerByte: 'Weight',
                transfer: 'Weight',
                call: 'Weight',
                callTransferSurcharge: 'Weight',
                callPerInputByte: 'Weight',
                callPerOutputByte: 'Weight',
                instantiate: 'Weight',
                instantiatePerInputByte: 'Weight',
                instantiatePerOutputByte: 'Weight',
                hashSha2256: 'Weight',
                hashSha2256PerByte: 'Weight',
                hashKeccak256: 'Weight',
                hashKeccak256PerByte: 'Weight',
                hashBlake2256: 'Weight',
                hashBlake2256PerByte: 'Weight',
                hashBlake2128: 'Weight',
                hashBlake2128PerByte: 'Weight'
            },
            HostFnWeights: {
                caller: 'Weight',
                address: 'Weight',
                gasLeft: 'Weight',
                balance: 'Weight',
                valueTransferred: 'Weight',
                minimumBalance: 'Weight',
                tombstoneDeposit: 'Weight',
                rentAllowance: 'Weight',
                blockNumber: 'Weight',
                now: 'Weight',
                weightToFee: 'Weight',
                gas: 'Weight',
                input: 'Weight',
                inputPerByte: 'Weight',
                return: 'Weight',
                returnPerByte: 'Weight',
                terminate: 'Weight',
                terminatePerCodeByte: 'Weight',
                restoreTo: 'Weight',
                restoreToPerCallerCodeByte: 'Weight',
                restoreToPerTombstoneCodeByte: 'Weight',
                restoreToPerDelta: 'Weight',
                random: 'Weight',
                depositEvent: 'Weight',
                depositEventPerTopic: 'Weight',
                depositEventPerByte: 'Weight',
                setRentAllowance: 'Weight',
                setStorage: 'Weight',
                setStoragePerByte: 'Weight',
                clearStorage: 'Weight',
                getStorage: 'Weight',
                getStoragePerByte: 'Weight',
                transfer: 'Weight',
                call: 'Weight',
                callPerCodeByte: 'Weight',
                callTransferSurcharge: 'Weight',
                callPerInputByte: 'Weight',
                callPerOutputByte: 'Weight',
                instantiate: 'Weight',
                instantiatePerCodeByte: 'Weight',
                instantiatePerInputByte: 'Weight',
                instantiatePerOutputByte: 'Weight',
                instantiatePerSaltByte: 'Weight',
                hashSha2256: 'Weight',
                hashSha2256PerByte: 'Weight',
                hashKeccak256: 'Weight',
                hashKeccak256PerByte: 'Weight',
                hashBlake2256: 'Weight',
                hashBlake2256PerByte: 'Weight',
                hashBlake2128: 'Weight',
                hashBlake2128PerByte: 'Weight',
                rentParams: 'Weight'
            },
            InstantiateRequestV1: {
                origin: 'AccountId',
                value: 'Balance',
                gasLimit: 'Gas',
                code: 'Bytes',
                data: 'Bytes',
                salt: 'Bytes'
            },
            InstantiateRequestV2: {
                _fallback: 'InstantiateRequestV1',
                origin: 'AccountId',
                value: 'Balance',
                gasLimit: 'Gas',
                storageDepositLimit: 'Option<Balance>',
                code: 'Bytes',
                data: 'Bytes',
                salt: 'Bytes'
            },
            InstantiateRequest: {
                _fallback: 'InstantiateRequestV2',
                origin: 'AccountId',
                value: 'Balance',
                gasLimit: 'Gas',
                storageDepositLimit: 'Option<Balance>',
                code: 'CodeSource',
                data: 'Bytes',
                salt: 'Bytes'
            },
            ContractInstantiateResultTo267: 'Result<InstantiateReturnValueTo267, Null>',
            ContractInstantiateResultTo299: 'Result<InstantiateReturnValueOk, Null>',
            ContractInstantiateResult: {
                gasConsumed: 'WeightV2',
                gasRequired: 'WeightV2',
                storageDeposit: 'StorageDeposit',
                debugMessage: 'Text',
                result: 'InstantiateReturnValue'
            },
            ContractInstantiateResultU64: {
                _fallback: 'ContractInstantiateResultTo299',
                gasConsumed: 'u64',
                gasRequired: 'u64',
                storageDeposit: 'StorageDeposit',
                debugMessage: 'Text',
                result: 'InstantiateReturnValue'
            },
            InstantiateReturnValueTo267: {
                result: 'ExecReturnValue',
                accountId: 'AccountId',
                rentProjection: 'Option<RentProjection>'
            },
            InstantiateReturnValueOk: {
                result: 'ExecReturnValue',
                accountId: 'AccountId'
            },
            InstantiateReturnValue: 'Result<InstantiateReturnValueOk, DispatchError>',
            InstructionWeights: {
                i64const: 'u32',
                i64load: 'u32',
                i64store: 'u32',
                select: 'u32',
                rIf: 'u32',
                br: 'u32',
                brIf: 'u32',
                brIable: 'u32',
                brIablePerEntry: 'u32',
                call: 'u32',
                callIndirect: 'u32',
                callIndirectPerParam: 'u32',
                localGet: 'u32',
                localSet: 'u32',
                local_tee: 'u32',
                globalGet: 'u32',
                globalSet: 'u32',
                memoryCurrent: 'u32',
                memoryGrow: 'u32',
                i64clz: 'u32',
                i64ctz: 'u32',
                i64popcnt: 'u32',
                i64eqz: 'u32',
                i64extendsi32: 'u32',
                i64extendui32: 'u32',
                i32wrapi64: 'u32',
                i64eq: 'u32',
                i64ne: 'u32',
                i64lts: 'u32',
                i64ltu: 'u32',
                i64gts: 'u32',
                i64gtu: 'u32',
                i64les: 'u32',
                i64leu: 'u32',
                i64ges: 'u32',
                i64geu: 'u32',
                i64add: 'u32',
                i64sub: 'u32',
                i64mul: 'u32',
                i64divs: 'u32',
                i64divu: 'u32',
                i64rems: 'u32',
                i64remu: 'u32',
                i64and: 'u32',
                i64or: 'u32',
                i64xor: 'u32',
                i64shl: 'u32',
                i64shrs: 'u32',
                i64shru: 'u32',
                i64rotl: 'u32',
                i64rotr: 'u32'
            },
            LimitsTo264: {
                eventTopics: 'u32',
                stackHeight: 'u32',
                globals: 'u32',
                parameters: 'u32',
                memoryPages: 'u32',
                tableSize: 'u32',
                brTableSize: 'u32',
                subjectLen: 'u32',
                codeSize: 'u32'
            },
            Limits: {
                eventTopics: 'u32',
                stackHeight: 'u32',
                globals: 'u32',
                parameters: 'u32',
                memoryPages: 'u32',
                tableSize: 'u32',
                brTableSize: 'u32',
                subjectLen: 'u32'
            },
            PrefabWasmModule: {
                scheduleVersion: 'Compact<u32>',
                initial: 'Compact<u32>',
                maximum: 'Compact<u32>',
                refcount: 'Compact<u64>',
                _reserved: 'Option<Null>',
                code: 'Bytes',
                originalCodeLen: 'u32'
            },
            RentProjection: {
                _enum: {
                    EvictionAt: 'BlockNumber',
                    NoEviction: 'Null'
                }
            },
            ScheduleTo212: {
                version: 'u32',
                putCodePerByteCost: 'Gas',
                growMemCost: 'Gas',
                regularOpCost: 'Gas',
                returnDataPerByteCost: 'Gas',
                eventDataPerByteCost: 'Gas',
                eventPerTopicCost: 'Gas',
                eventBaseCost: 'Gas',
                sandboxDataReadCost: 'Gas',
                sandboxDataWriteCost: 'Gas',
                maxEventTopics: 'u32',
                maxStackHeight: 'u32',
                maxMemoryPages: 'u32',
                enablePrintln: 'bool',
                maxSubjectLen: 'u32'
            },
            ScheduleTo258: {
                version: 'u32',
                putCodePerByteCost: 'Gas',
                growMemCost: 'Gas',
                regularOpCost: 'Gas',
                returnDataPerByteCost: 'Gas',
                eventDataPerByteCost: 'Gas',
                eventPerTopicCost: 'Gas',
                eventBaseCost: 'Gas',
                sandboxDataReadCost: 'Gas',
                sandboxDataWriteCost: 'Gas',
                transferCost: 'Gas',
                maxEventTopics: 'u32',
                maxStackHeight: 'u32',
                maxMemoryPages: 'u32',
                enablePrintln: 'bool',
                maxSubjectLen: 'u32'
            },
            ScheduleTo264: {
                version: 'u32',
                enablePrintln: 'bool',
                limits: 'LimitsTo264',
                instructionWeights: 'InstructionWeights',
                hostFnWeights: 'HostFnWeightsTo264'
            },
            Schedule: {
                version: 'u32',
                enablePrintln: 'bool',
                limits: 'Limits',
                instructionWeights: 'InstructionWeights',
                hostFnWeights: 'HostFnWeights'
            },
            SeedOf: 'Hash',
            StorageDeposit: {
                _enum: {
                    Refund: 'Balance',
                    Charge: 'Balance'
                }
            },
            TombstoneContractInfo: 'Hash',
            TrieId: 'Bytes'
        }
    };

    const AllConvictions = [
        'None',
        'Locked1x',
        'Locked2x',
        'Locked3x',
        'Locked4x',
        'Locked5x',
        'Locked6x'
    ];
    const definitions$_ = {
        rpc: {},
        types: {
            AccountVote: {
                _enum: {
                    Standard: 'AccountVoteStandard',
                    Split: 'AccountVoteSplit'
                }
            },
            AccountVoteSplit: {
                aye: 'Balance',
                nay: 'Balance'
            },
            AccountVoteStandard: {
                vote: 'Vote',
                balance: 'Balance'
            },
            Conviction: {
                _enum: AllConvictions
            },
            Delegations: {
                votes: 'Balance',
                capital: 'Balance'
            },
            PreimageStatus: {
                _enum: {
                    Missing: 'BlockNumber',
                    Available: 'PreimageStatusAvailable'
                }
            },
            PreimageStatusAvailable: {
                data: 'Bytes',
                provider: 'AccountId',
                deposit: 'Balance',
                since: 'BlockNumber',
                expiry: 'Option<BlockNumber>'
            },
            PriorLock: '(BlockNumber, Balance)',
            PropIndex: 'u32',
            Proposal: 'Call',
            ProxyState: {
                _enum: {
                    Open: 'AccountId',
                    Active: 'AccountId'
                }
            },
            ReferendumIndex: 'u32',
            ReferendumInfoTo239: {
                end: 'BlockNumber',
                proposalHash: 'Hash',
                threshold: 'VoteThreshold',
                delay: 'BlockNumber'
            },
            ReferendumInfo: {
                _enum: {
                    Ongoing: 'ReferendumStatus',
                    Finished: 'ReferendumInfoFinished'
                }
            },
            ReferendumInfoFinished: {
                approved: 'bool',
                end: 'BlockNumber'
            },
            ReferendumStatus: {
                end: 'BlockNumber',
                proposalHash: 'Hash',
                threshold: 'VoteThreshold',
                delay: 'BlockNumber',
                tally: 'Tally'
            },
            Tally: {
                ayes: 'Balance',
                nays: 'Balance',
                turnout: 'Balance'
            },
            Voting: {
                _enum: {
                    Direct: 'VotingDirect',
                    Delegating: 'VotingDelegating'
                }
            },
            VotingDirect: {
                votes: 'Vec<VotingDirectVote>',
                delegations: 'Delegations',
                prior: 'PriorLock'
            },
            VotingDirectVote: '(ReferendumIndex, AccountVote)',
            VotingDelegating: {
                balance: 'Balance',
                target: 'AccountId',
                conviction: 'Conviction',
                delegations: 'Delegations',
                prior: 'PriorLock'
            }
        }
    };

    const rpc$d = {
        getBlockStats: {
            description: 'Reexecute the specified `block_hash` and gather statistics while doing so',
            isUnsafe: true,
            params: [
                {
                    isHistoric: true,
                    name: 'at',
                    type: 'Hash'
                }
            ],
            type: 'Option<BlockStats>'
        }
    };

    const definitions$Z = {
        rpc: rpc$d,
        types: {
            BlockStats: {
                witnessLen: 'u64',
                witnessCompactLen: 'u64',
                blockLen: 'u64',
                blockNumExtrinsics: 'u64'
            }
        }
    };

    const runtime$p = {
        AuthorityDiscoveryApi: [
            {
                methods: {
                    authorities: {
                        description: 'Retrieve authority identifiers of the current and next authority set.',
                        params: [],
                        type: 'Vec<AuthorityId>'
                    }
                },
                version: 1
            }
        ]
    };

    const definitions$Y = {
        rpc: {},
        runtime: runtime$p,
        types: {}
    };

    const definitions$X = {
        rpc: {},
        types: {
            ApprovalFlag: 'u32',
            DefunctVoter: {
                who: 'AccountId',
                voteCount: 'Compact<u32>',
                candidateCount: 'Compact<u32>'
            },
            Renouncing: {
                _enum: {
                    Member: 'Null',
                    RunnerUp: 'Null',
                    Candidate: 'Compact<u32>'
                }
            },
            SetIndex: 'u32',
            Vote: 'GenericVote',
            VoteIndex: 'u32',
            VoterInfo: {
                lastActive: 'VoteIndex',
                lastWin: 'VoteIndex',
                pot: 'Balance',
                stake: 'Balance'
            },
            VoteThreshold: {
                _enum: [
                    'Super Majority Approve',
                    'Super Majority Against',
                    'Simple Majority'
                ]
            }
        }
    };

    const rpc$c = {
        createBlock: {
            description: 'Instructs the manual-seal authorship task to create a new block',
            params: [
                {
                    name: 'createEmpty',
                    type: 'bool'
                },
                {
                    name: 'finalize',
                    type: 'bool'
                },
                {
                    isOptional: true,
                    name: 'parentHash',
                    type: 'BlockHash'
                }
            ],
            type: 'CreatedBlock'
        },
        finalizeBlock: {
            description: 'Instructs the manual-seal authorship task to finalize a block',
            params: [
                {
                    name: 'hash',
                    type: 'BlockHash'
                },
                {
                    isOptional: true,
                    name: 'justification',
                    type: 'Justification'
                }
            ],
            type: 'bool'
        }
    };

    const definitions$W = {
        rpc: rpc$c,
        types: {
            CreatedBlock: {
                _alias: {
                    blockHash: 'hash'
                },
                blockHash: 'BlockHash',
                aux: 'ImportedAux'
            },
            ImportedAux: {
                headerOnly: 'bool',
                clearJustificationRequests: 'bool',
                needsJustification: 'bool',
                badJustification: 'bool',
                needsFinalityProof: 'bool',
                isNewBest: 'bool'
            }
        }
    };

    const definitions$V = {
        rpc: {},
        types: {
            EvmAccount: {
                nonce: 'u256',
                balance: 'u256'
            },
            EvmCallInfo: {
                exitReason: 'ExitReason',
                value: 'Bytes',
                usedGas: 'U256',
                logs: 'Vec<EvmLog>'
            },
            EvmCreateInfo: {
                exitReason: 'ExitReason',
                value: 'H160',
                usedGas: 'U256',
                logs: 'Vec<EvmLog>'
            },
            EvmCallInfoV2: {
                exitReason: 'ExitReason',
                value: 'Bytes',
                usedGas: 'U256',
                weightInfo: 'Option<EvmWeightInfo>',
                logs: 'Vec<EvmLog>'
            },
            EvmCreateInfoV2: {
                exitReason: 'ExitReason',
                value: 'H160',
                usedGas: 'U256',
                weightInfo: 'Option<EvmWeightInfo>',
                logs: 'Vec<EvmLog>'
            },
            EvmLog: {
                address: 'H160',
                topics: 'Vec<H256>',
                data: 'Bytes'
            },
            EvmVicinity: {
                gasPrice: 'u256',
                origin: 'H160'
            },
            EvmWeightInfo: {
                refTimeLimit: 'Option<u64>',
                proofSizeLimit: 'Option<u64>',
                refTimeUsage: 'Option<u64>',
                proofSizeUsage: 'Option<u64>'
            },
            ExitError: {
                _enum: {
                    StackUnderflow: 'Null',
                    StackOverflow: 'Null',
                    InvalidJump: 'Null',
                    InvalidRange: 'Null',
                    DesignatedInvalid: 'Null',
                    CallTooDeep: 'Null',
                    CreateCollision: 'Null',
                    CreateContractLimit: 'Null',
                    OutOfOffset: 'Null',
                    OutOfGas: 'Null',
                    OutOfFund: 'Null',
                    PCUnderflow: 'Null',
                    CreateEmpty: 'Null',
                    Other: 'Text'
                }
            },
            ExitFatal: {
                _enum: {
                    NotSupported: 'Null',
                    UnhandledInterrupt: 'Null',
                    CallErrorAsFatal: 'ExitError',
                    Other: 'Text'
                }
            },
            ExitReason: {
                _enum: {
                    Succeed: 'ExitSucceed',
                    Error: 'ExitError',
                    Revert: 'ExitRevert',
                    Fatal: 'ExitFatal'
                }
            },
            ExitRevert: {
                _enum: ['Reverted']
            },
            ExitSucceed: {
                _enum: ['Stopped', 'Returned', 'Suicided']
            }
        }
    };

    const definitions$U = {
        rpc: {},
        types: {
            Extrinsic: 'GenericExtrinsic',
            ExtrinsicEra: 'GenericExtrinsicEra',
            ExtrinsicPayload: 'GenericExtrinsicPayload',
            ExtrinsicSignature: 'MultiSignature',
            ExtrinsicV4: 'GenericExtrinsicV4',
            ExtrinsicPayloadV4: 'GenericExtrinsicPayloadV4',
            ExtrinsicSignatureV4: 'GenericExtrinsicSignatureV4',
            ExtrinsicUnknown: 'GenericExtrinsicUnknown',
            ExtrinsicPayloadUnknown: 'GenericExtrinsicPayloadUnknown',
            ExtrinsicV5: 'GenericExtrinsicV5',
            ExtrinsicPayloadV5: 'GenericExtrinsicPayloadV5',
            ExtrinsicSignatureV5: 'GenericExtrinsicSignatureV5',
            Era: 'ExtrinsicEra',
            ImmortalEra: 'GenericImmortalEra',
            MortalEra: 'GenericMortalEra',
            AnySignature: 'H512',
            MultiSignature: {
                _enum: {
                    Ed25519: 'Ed25519Signature',
                    Sr25519: 'Sr25519Signature',
                    Ecdsa: 'EcdsaSignature'
                }
            },
            Signature: 'H512',
            SignerPayload: 'GenericSignerPayload',
            EcdsaSignature: '[u8; 65]',
            Ed25519Signature: 'H512',
            Sr25519Signature: 'H512'
        }
    };

    const runtime$o = {
        FungiblesApi: [
            {
                methods: {
                    query_account_balances: {
                        description: 'Returns the list of all `MultiAsset` that an `AccountId` has',
                        params: [
                            {
                                name: 'account',
                                type: 'AccountId'
                            }
                        ],
                        type: 'Result<Vec<XcmV3MultiAsset>, FungiblesAccessError>'
                    }
                },
                version: 1
            },
            {
                methods: {
                    query_account_balances: {
                        description: 'Returns the list of all `MultiAsset` that an `AccountId` has',
                        params: [
                            {
                                name: 'account',
                                type: 'AccountId'
                            }
                        ],
                        type: 'Result<XcmVersionedAssets, FungiblesAccessError>'
                    }
                },
                version: 2
            }
        ]
    };

    const definitions$T = {
        rpc: {},
        runtime: runtime$o,
        types: {
            FungiblesAccessError: {
                _enum: ['AssetIdConversionFailed', 'AmountToBalanceConversionFailed']
            }
        }
    };

    const definitions$S = {
        rpc: {},
        types: {
            AssetOptions: {
                initalIssuance: 'Compact<Balance>',
                permissions: 'PermissionLatest'
            },
            Owner: {
                _enum: {
                    None: 'Null',
                    Address: 'AccountId'
                }
            },
            PermissionsV1: {
                update: 'Owner',
                mint: 'Owner',
                burn: 'Owner'
            },
            PermissionVersions: {
                _enum: {
                    V1: 'PermissionsV1'
                }
            },
            PermissionLatest: 'PermissionsV1'
        }
    };

    const runtime$n = {
        GenesisBuilder: [
            {
                methods: {
                    build_config: {
                        description: 'Build `RuntimeGenesisConfig` from a JSON blob not using any defaults and store it in the storage.',
                        params: [
                            {
                                name: 'json',
                                type: 'Vec<u8>'
                            }
                        ],
                        type: 'Result<(), GenesisBuildErr>'
                    },
                    create_default_config: {
                        description: 'Creates the default `RuntimeGenesisConfig` and returns it as a JSON blob.',
                        params: [],
                        type: 'Vec<u8>'
                    }
                },
                version: 1
            }
        ]
    };

    const definitions$R = {
        rpc: {},
        runtime: runtime$n,
        types: {
            GenesisBuildErr: 'Text'
        }
    };

    const definitions$Q = {
        rpc: {},
        types: {
            ActiveGilt: {
                proportion: 'Perquintill',
                amount: 'Balance',
                who: 'AccountId',
                expiry: 'BlockNumber'
            },
            ActiveGiltsTotal: {
                frozen: 'Balance',
                proportion: 'Perquintill',
                index: 'ActiveIndex',
                target: 'Perquintill'
            },
            ActiveIndex: 'u32',
            GiltBid: {
                amount: 'Balance',
                who: 'AccountId'
            }
        }
    };

    const rpc$b = {
        proveFinality: {
            description: 'Prove finality for the given block number, returning the Justification for the last block in the set.',
            params: [
                {
                    name: 'blockNumber',
                    type: 'BlockNumber'
                }
            ],
            type: 'Option<EncodedFinalityProofs>'
        },
        roundState: {
            description: 'Returns the state of the current best round state as well as the ongoing background rounds',
            params: [],
            type: 'ReportedRoundStates'
        },
        subscribeJustifications: {
            description: 'Subscribes to grandpa justifications',
            params: [],
            pubsub: [
                'justifications',
                'subscribeJustifications',
                'unsubscribeJustifications'
            ],
            type: 'JustificationNotification'
        }
    };

    const GRANDPA_V2_V3 = {
        generate_key_ownership_proof: {
            description: 'Generates a proof of key ownership for the given authority in the given set.',
            params: [
                {
                    name: 'setId',
                    type: 'SetId'
                },
                {
                    name: 'authorityId',
                    type: 'AuthorityId'
                }
            ],
            type: 'Option<OpaqueKeyOwnershipProof>'
        },
        grandpa_authorities: {
            description: 'Get the current GRANDPA authorities and weights. This should not change except for when changes are scheduled and the corresponding delay has passed.',
            params: [],
            type: 'AuthorityList'
        },
        submit_report_equivocation_unsigned_extrinsic: {
            description: 'Submits an unsigned extrinsic to report an equivocation.',
            params: [
                {
                    name: 'equivocationProof',
                    type: 'GrandpaEquivocationProof'
                },
                {
                    name: 'keyOwnerProof',
                    type: 'OpaqueKeyOwnershipProof'
                }
            ],
            type: 'Option<Null>'
        }
    };
    const runtime$m = {
        GrandpaApi: [
            {
                methods: {
                    current_set_id: {
                        description: 'Get current GRANDPA authority set id.',
                        params: [],
                        type: 'SetId'
                    },
                    ...GRANDPA_V2_V3
                },
                version: 3
            },
            {
                methods: GRANDPA_V2_V3,
                version: 2
            }
        ]
    };

    const definitions$P = {
        rpc: rpc$b,
        runtime: runtime$m,
        types: {
            AuthorityIndex: 'u64',
            AuthorityList: 'Vec<NextAuthority>',
            AuthoritySet: {
                currentAuthorities: 'AuthorityList',
                setId: 'u64',
                pendingStandardChanges: 'ForkTreePendingChange',
                pendingForcedChanges: 'Vec<PendingChange>',
                authoritySetChanges: 'AuthoritySetChanges'
            },
            ForkTreePendingChange: {
                roots: 'Vec<ForkTreePendingChangeNode>',
                bestFinalizedNumber: 'Option<BlockNumber>'
            },
            ForkTreePendingChangeNode: {
                hash: 'BlockHash',
                number: 'BlockNumber',
                data: 'PendingChange',
                children: 'Vec<ForkTreePendingChangeNode>'
            },
            AuthoritySetChange: '(U64, BlockNumber)',
            AuthoritySetChanges: 'Vec<AuthoritySetChange>',
            AuthorityWeight: 'u64',
            DelayKind: {
                _enum: {
                    Finalized: 'Null',
                    Best: 'DelayKindBest'
                }
            },
            DelayKindBest: {
                medianLastFinalized: 'BlockNumber'
            },
            EncodedFinalityProofs: 'Bytes',
            GrandpaEquivocation: {
                _enum: {
                    Prevote: 'GrandpaEquivocationValue',
                    Precommit: 'GrandpaEquivocationValue'
                }
            },
            GrandpaEquivocationProof: {
                setId: 'SetId',
                equivocation: 'GrandpaEquivocation'
            },
            GrandpaEquivocationValue: {
                roundNumber: 'u64',
                identity: 'AuthorityId',
                first: '(GrandpaPrevote, AuthoritySignature)',
                second: '(GrandpaPrevote, AuthoritySignature)'
            },
            GrandpaPrevote: {
                targetHash: 'Hash',
                targetNumber: 'BlockNumber'
            },
            GrandpaCommit: {
                targetHash: 'BlockHash',
                targetNumber: 'BlockNumber',
                precommits: 'Vec<GrandpaSignedPrecommit>'
            },
            GrandpaPrecommit: {
                targetHash: 'BlockHash',
                targetNumber: 'BlockNumber'
            },
            GrandpaSignedPrecommit: {
                precommit: 'GrandpaPrecommit',
                signature: 'AuthoritySignature',
                id: 'AuthorityId'
            },
            GrandpaJustification: {
                round: 'u64',
                commit: 'GrandpaCommit',
                votesAncestries: 'Vec<Header>'
            },
            JustificationNotification: 'Bytes',
            KeyOwnerProof: 'MembershipProof',
            NextAuthority: '(AuthorityId, AuthorityWeight)',
            PendingChange: {
                nextAuthorities: 'AuthorityList',
                delay: 'BlockNumber',
                canonHeight: 'BlockNumber',
                canonHash: 'BlockHash',
                delayKind: 'DelayKind'
            },
            PendingPause: {
                scheduledAt: 'BlockNumber',
                delay: 'BlockNumber'
            },
            PendingResume: {
                scheduledAt: 'BlockNumber',
                delay: 'BlockNumber'
            },
            Precommits: {
                currentWeight: 'u32',
                missing: 'BTreeSet<AuthorityId>'
            },
            Prevotes: {
                currentWeight: 'u32',
                missing: 'BTreeSet<AuthorityId>'
            },
            ReportedRoundStates: {
                setId: 'u32',
                best: 'RoundState',
                background: 'Vec<RoundState>'
            },
            RoundState: {
                round: 'u32',
                totalWeight: 'u32',
                thresholdWeight: 'u32',
                prevotes: 'Prevotes',
                precommits: 'Precommits'
            },
            SetId: 'u64',
            StoredPendingChange: {
                scheduledAt: 'BlockNumber',
                delay: 'BlockNumber',
                nextAuthorities: 'AuthorityList'
            },
            StoredState: {
                _enum: {
                    Live: 'Null',
                    PendingPause: 'PendingPause',
                    Paused: 'Null',
                    PendingResume: 'PendingResume'
                }
            }
        }
    };

    const definitions$O = {
        rpc: {},
        types: {
            IdentityFields: {
                _set: {
                    _bitLength: 64,
                    Display: 0b00000000_00000000_00000000_0000_0001,
                    Legal: 0b00000000_00000000_00000000_0000_0010,
                    Web: 0b00000000_00000000_00000000_0000_0100,
                    Riot: 0b00000000_00000000_00000000_0000_1000,
                    Email: 0b00000000_00000000_00000000_0001_0000,
                    PgpFingerprint: 0b00000000_00000000_00000000_0010_0000,
                    Image: 0b00000000_00000000_00000000_0100_0000,
                    Twitter: 0b00000000_00000000_00000000_1000_0000
                }
            },
            IdentityInfoAdditional: '(Data, Data)',
            IdentityInfoTo198: {
                additional: 'Vec<IdentityInfoAdditional>',
                display: 'Data',
                legal: 'Data',
                web: 'Data',
                riot: 'Data',
                email: 'Data',
                pgpFingerprint: 'Option<H160>',
                image: 'Data'
            },
            IdentityInfo: {
                _fallback: 'IdentityInfoTo198',
                additional: 'Vec<IdentityInfoAdditional>',
                display: 'Data',
                legal: 'Data',
                web: 'Data',
                riot: 'Data',
                email: 'Data',
                pgpFingerprint: 'Option<H160>',
                image: 'Data',
                twitter: 'Data'
            },
            IdentityJudgement: {
                _enum: {
                    Unknown: 'Null',
                    FeePaid: 'Balance',
                    Reasonable: 'Null',
                    KnownGood: 'Null',
                    OutOfDate: 'Null',
                    LowQuality: 'Null',
                    Erroneous: 'Null'
                }
            },
            RegistrationJudgement: '(RegistrarIndex, IdentityJudgement)',
            RegistrationTo198: {
                judgements: 'Vec<RegistrationJudgement>',
                deposit: 'Balance',
                info: 'IdentityInfoTo198'
            },
            Registration: {
                _fallback: 'RegistrationTo198',
                judgements: 'Vec<RegistrationJudgement>',
                deposit: 'Balance',
                info: 'IdentityInfo'
            },
            RegistrarIndex: 'u32',
            RegistrarInfo: {
                account: 'AccountId',
                fee: 'Balance',
                fields: 'IdentityFields'
            }
        }
    };

    const definitions$N = {
        rpc: {},
        types: {
            AuthIndex: 'u32',
            AuthoritySignature: 'Signature',
            Heartbeat: {
                blockNumber: 'BlockNumber',
                networkState: 'OpaqueNetworkState',
                sessionIndex: 'SessionIndex',
                authorityIndex: 'AuthIndex',
                validatorsLen: 'u32'
            },
            HeartbeatTo244: {
                blockNumber: 'BlockNumber',
                networkState: 'OpaqueNetworkState',
                sessionIndex: 'SessionIndex',
                authorityIndex: 'AuthIndex'
            },
            OpaqueMultiaddr: 'Opaque<Bytes>',
            OpaquePeerId: 'Opaque<Bytes>',
            OpaqueNetworkState: {
                peerId: 'OpaquePeerId',
                externalAddresses: 'Vec<OpaqueMultiaddr>'
            }
        }
    };

    const definitions$M = {
        rpc: {},
        types: {
            CallIndex: '(u8, u8)',
            LotteryConfig: {
                price: 'Balance',
                start: 'BlockNumber',
                length: 'BlockNumber',
                delay: 'BlockNumber',
                repeat: 'bool'
            }
        }
    };

    const runtime$l = {
        MixnetApi: [
            {
                methods: {
                    current_mixnodes: {
                        description: 'Get the index and phase of the current session.',
                        params: [],
                        type: 'Result<Mixnode, MixnodesErr>'
                    },
                    maybe_register: {
                        description: 'Try to register a mixnode for the next session.',
                        params: [
                            {
                                name: 'session_index',
                                type: 'u32'
                            },
                            {
                                name: 'mixnode',
                                type: 'Mixnode'
                            }
                        ],
                        type: 'bool'
                    },
                    prev_mixnodes: {
                        description: 'Get the index and phase of the current session.',
                        params: [],
                        type: 'Result<Mixnode, MixnodesErr>'
                    },
                    session_status: {
                        description: 'Get the index and phase of the current session.',
                        params: [],
                        type: 'SessionStatus'
                    }
                },
                version: 1
            }
        ]
    };

    const definitions$L = {
        rpc: {},
        runtime: runtime$l,
        types: {
            Mixnode: {
                externalAddresses: 'Vec<Bytes>',
                kxPublic: '[u8; 32]',
                peerId: '[u8; 32]'
            },
            MixnodesErr: {
                _enum: {
                    InsufficientRegistrations: {
                        min: 'u32',
                        num: 'u32'
                    }
                }
            },
            SessionPhase: {
                _enum: ['CoverToCurrent', 'RequestsToCurrent', 'CoverToPrev', 'DisconnectFromPrev']
            },
            SessionStatus: {
                currentIndex: 'u32',
                phase: 'SessionPhase'
            }
        }
    };

    const rpc$a = {
        generateProof: {
            description: 'Generate MMR proof for the given block numbers.',
            params: [
                {
                    name: 'blockNumbers',
                    type: 'Vec<u64>'
                },
                {
                    isOptional: true,
                    name: 'bestKnownBlockNumber',
                    type: 'u64'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'MmrLeafBatchProof'
        },
        root: {
            description: 'Get the MMR root hash for the current best block.',
            params: [
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'MmrHash'
        },
        verifyProof: {
            description: 'Verify an MMR proof',
            params: [
                {
                    name: 'proof',
                    type: 'MmrLeafBatchProof'
                }
            ],
            type: 'bool'
        },
        verifyProofStateless: {
            description: 'Verify an MMR proof statelessly given an mmr_root',
            params: [
                {
                    name: 'root',
                    type: 'MmrHash'
                },
                {
                    name: 'proof',
                    type: 'MmrLeafBatchProof'
                }
            ],
            type: 'bool'
        }
    };

    const MMR_V2 = {
        generate_proof: {
            description: 'Generate MMR proof for the given block numbers.',
            params: [
                {
                    name: 'blockNumbers',
                    type: 'Vec<BlockNumber>'
                },
                {
                    name: 'bestKnownBlockNumber',
                    type: 'Option<BlockNumber>'
                }
            ],
            type: 'Result<(Vec<MmrEncodableOpaqueLeaf>, MmrBatchProof), MmrError>'
        },
        mmr_leaf_count: {
            description: 'Return the number of MMR blocks in the chain.',
            params: [],
            type: 'Result<U64, MmrError>'
        },
        mmr_root: {
            description: 'Return the on-chain MMR root hash.',
            params: [],
            type: 'Result<Hash, MmrError>'
        },
        verify_proof: {
            description: 'Verify MMR proof against on-chain MMR.',
            params: [
                {
                    name: 'leaves',
                    type: 'Vec<MmrEncodableOpaqueLeaf>'
                },
                {
                    name: 'proof',
                    type: 'MmrBatchProof'
                }
            ],
            type: 'Result<(), MmrError>'
        },
        verify_proof_stateless: {
            description: 'Verify MMR proof against given root hash.',
            params: [
                {
                    name: 'root',
                    type: 'Hash'
                },
                {
                    name: 'leaves',
                    type: 'Vec<MmrEncodableOpaqueLeaf>'
                },
                {
                    name: 'proof',
                    type: 'MmrBatchProof'
                }
            ],
            type: 'Result<(), MmrError>'
        }
    };
    const MMR_V1 = {
        generate_batch_proof: {
            description: 'Generate MMR proof for a series of leaves under given indices.',
            params: [
                {
                    name: 'leafIndices',
                    type: 'Vec<MmrLeafIndex>'
                }
            ],
            type: 'Result<(Vec<MmrEncodableOpaqueLeaf>, MmrBatchProof), MmrError>'
        },
        generate_proof: {
            description: 'Generate MMR proof for a leaf under given index.',
            params: [
                {
                    name: 'leafIndex',
                    type: 'MmrLeafIndex'
                }
            ],
            type: 'Result<(MmrEncodableOpaqueLeaf, MmrProof), MmrError>'
        },
        mmr_root: {
            description: 'Return the on-chain MMR root hash.',
            params: [],
            type: 'Result<Hash, MmrError>'
        },
        verify_batch_proof: {
            description: 'Verify MMR proof against on-chain MMR for a batch of leaves.',
            params: [
                {
                    name: 'leaves',
                    type: 'Vec<MmrEncodableOpaqueLeaf>'
                },
                {
                    name: 'proof',
                    type: 'MmrBatchProof'
                }
            ],
            type: 'Result<(), MmrError>'
        },
        verify_batch_proof_stateless: {
            description: 'Verify MMR proof against given root hash or a batch of leaves.',
            params: [
                {
                    name: 'root',
                    type: 'Hash'
                },
                {
                    name: 'leaves',
                    type: 'Vec<MmrEncodableOpaqueLeaf>'
                },
                {
                    name: 'proof',
                    type: 'MmrBatchProof'
                }
            ],
            type: 'Result<(), MmrError>'
        },
        verify_proof: {
            description: 'Verify MMR proof against on-chain MMR.',
            params: [
                {
                    name: 'leaf',
                    type: 'MmrEncodableOpaqueLeaf'
                },
                {
                    name: 'proof',
                    type: 'MmrProof'
                }
            ],
            type: 'Result<(), MmrError>'
        },
        verify_proof_stateless: {
            description: 'Verify MMR proof against given root hash.',
            params: [
                {
                    name: 'root',
                    type: 'Hash'
                },
                {
                    name: 'leaf',
                    type: 'MmrEncodableOpaqueLeaf'
                },
                {
                    name: 'proof',
                    type: 'MmrProof'
                }
            ],
            type: 'Result<(), MmrError>'
        }
    };
    const runtime$k = {
        MmrApi: [
            {
                methods: MMR_V2,
                version: 2
            },
            {
                methods: MMR_V1,
                version: 1
            }
        ]
    };

    const definitions$K = {
        rpc: rpc$a,
        runtime: runtime$k,
        types: {
            MmrBatchProof: {
                leafIndices: 'Vec<MmrLeafIndex>',
                leafCount: 'MmrNodeIndex',
                items: 'Vec<Hash>'
            },
            MmrEncodableOpaqueLeaf: 'Bytes',
            MmrError: {
                _enum: ['InvalidNumericOp', 'Push', 'GetRoot', 'Commit', 'GenerateProof', 'Verify', 'LeafNotFound', ' PalletNotIncluded', 'InvalidLeafIndex', 'InvalidBestKnownBlock']
            },
            MmrHash: 'Hash',
            MmrLeafBatchProof: {
                blockHash: 'BlockHash',
                leaves: 'Bytes',
                proof: 'Bytes'
            },
            MmrLeafIndex: 'u64',
            MmrLeafProof: {
                blockHash: 'BlockHash',
                leaf: 'Bytes',
                proof: 'Bytes'
            },
            MmrNodeIndex: 'u64',
            MmrProof: {
                leafIndex: 'MmrLeafIndex',
                leafCount: 'MmrNodeIndex',
                items: 'Vec<Hash>'
            }
        }
    };

    const runtime$j = {
        NftsApi: [
            {
                methods: {
                    attribute: {
                        description: 'An attribute',
                        params: [
                            {
                                name: 'collection',
                                type: 'NftCollectionId'
                            },
                            {
                                name: 'item',
                                type: 'NftItemId'
                            },
                            {
                                name: 'key',
                                type: 'Bytes'
                            }
                        ],
                        type: 'Option<Bytes>'
                    },
                    collection_attribute: {
                        description: 'A collection attribute',
                        params: [
                            {
                                name: 'collection',
                                type: 'NftCollectionId'
                            },
                            {
                                name: 'key',
                                type: 'Bytes'
                            }
                        ],
                        type: 'Option<Bytes>'
                    },
                    collection_owner: {
                        description: 'A collection owner',
                        params: [
                            {
                                name: 'collection',
                                type: 'NftCollectionId'
                            }
                        ],
                        type: 'Option<AccountId>'
                    },
                    custom_attribute: {
                        description: 'A custom attribute',
                        params: [
                            {
                                name: 'account',
                                type: 'AccountId'
                            },
                            {
                                name: 'collection',
                                type: 'NftCollectionId'
                            },
                            {
                                name: 'item',
                                type: 'NftItemId'
                            },
                            {
                                name: 'key',
                                type: 'Bytes'
                            }
                        ],
                        type: 'Option<Bytes>'
                    },
                    owner: {
                        description: 'Collection owner',
                        params: [
                            {
                                name: 'collection',
                                type: 'NftCollectionId'
                            },
                            {
                                name: 'item',
                                type: 'NftItemId'
                            }
                        ],
                        type: 'Option<AccountId>'
                    },
                    system_attribute: {
                        description: 'System attribute',
                        params: [
                            {
                                name: 'collection',
                                type: 'NftCollectionId'
                            },
                            {
                                name: 'item',
                                type: 'NftItemId'
                            },
                            {
                                name: 'key',
                                type: 'Bytes'
                            }
                        ],
                        type: 'Option<Bytes>'
                    }
                },
                version: 1
            }
        ]
    };

    const definitions$J = {
        rpc: {},
        runtime: runtime$j,
        types: {
            NftCollectionId: 'u32',
            NftItemId: 'u32'
        }
    };

    const runtime$i = {
        NominationPoolsApi: [
            {
                methods: {
                    balance_to_points: {
                        description: 'Returns the equivalent points of `new_funds` for a given pool.',
                        params: [
                            {
                                name: 'poolId',
                                type: 'NpPoolId'
                            },
                            {
                                name: 'newFunds',
                                type: 'Balance'
                            }
                        ],
                        type: 'Balance'
                    },
                    pending_rewards: {
                        description: 'Returns the pending rewards for the given member.',
                        params: [
                            {
                                name: 'member',
                                type: 'AccountId'
                            }
                        ],
                        type: 'Balance'
                    },
                    points_to_balance: {
                        description: 'Returns the equivalent balance of `points` for a given pool.',
                        params: [
                            {
                                name: 'poolId',
                                type: 'NpPoolId'
                            },
                            {
                                name: 'points',
                                type: 'Balance'
                            }
                        ],
                        type: 'Balance'
                    }
                },
                version: 1
            }
        ]
    };

    const definitions$I = {
        rpc: {},
        runtime: runtime$i,
        types: {
            NpApiError: {
                _enum: ['MemberNotFound', 'OverflowInPendingRewards']
            },
            NpPoolId: 'u32'
        }
    };

    const definitions$H = {
        rpc: {},
        types: {
            DeferredOffenceOf: '(Vec<OffenceDetails>, Vec<Perbill>, SessionIndex)',
            Kind: '[u8; 16]',
            OffenceDetails: {
                offender: 'Offender',
                reporters: 'Vec<Reporter>'
            },
            Offender: 'IdentificationTuple',
            OpaqueTimeSlot: 'Bytes',
            ReportIdOf: 'Hash',
            Reporter: 'AccountId'
        }
    };

    const runtime$h = {
        DifficultyApi: [
            {
                methods: {
                    difficulty: {
                        description: 'Return the target difficulty of the next block.',
                        params: [],
                        type: 'Raw'
                    }
                },
                version: 1
            }
        ],
        TimestampApi: [
            {
                methods: {
                    timestamp: {
                        description: 'API necessary for timestamp-based difficulty adjustment algorithms.',
                        params: [],
                        type: 'Moment'
                    }
                },
                version: 1
            }
        ]
    };

    const definitions$G = {
        rpc: {},
        runtime: runtime$h,
        types: {}
    };

    const definitions$F = {
        rpc: {},
        types: {
            ProxyDefinition: {
                delegate: 'AccountId',
                proxyType: 'ProxyType',
                delay: 'BlockNumber'
            },
            ProxyType: {
                _enum: ['Any', 'NonTransfer', 'Governance', 'Staking']
            },
            ProxyAnnouncement: {
                real: 'AccountId',
                callHash: 'Hash',
                height: 'BlockNumber'
            }
        }
    };

    const definitions$E = {
        rpc: {},
        types: {
            ActiveRecovery: {
                created: 'BlockNumber',
                deposit: 'Balance',
                friends: 'Vec<AccountId>'
            },
            RecoveryConfig: {
                delayPeriod: 'BlockNumber',
                deposit: 'Balance',
                friends: 'Vec<AccountId>',
                threshold: 'u16'
            }
        }
    };

    const definitions$D = {
        rpc: {},
        types: {
            Period: '(BlockNumber, u32)',
            Priority: 'u8',
            SchedulePeriod: 'Period',
            SchedulePriority: 'Priority',
            Scheduled: {
                maybeId: 'Option<Bytes>',
                priority: 'SchedulePriority',
                call: 'Call',
                maybePeriodic: 'Option<SchedulePeriod>',
                origin: 'PalletsOrigin'
            },
            ScheduledTo254: {
                maybeId: 'Option<Bytes>',
                priority: 'SchedulePriority',
                call: 'Call',
                maybePeriodic: 'Option<SchedulePeriod>'
            },
            TaskAddress: '(BlockNumber, u32)'
        }
    };

    const runtime$g = {
        SessionKeys: [
            {
                methods: {
                    decode_session_keys: {
                        description: 'Decode the given public session keys.',
                        params: [
                            {
                                name: 'encoded',
                                type: 'Bytes'
                            }
                        ],
                        type: 'Option<Vec<(Bytes, KeyTypeId)>>'
                    },
                    generate_session_keys: {
                        description: 'Generate a set of session keys with optionally using the given seed.',
                        params: [
                            {
                                name: 'seed',
                                type: 'Option<Bytes>'
                            }
                        ],
                        type: 'Bytes'
                    }
                },
                version: 1
            }
        ]
    };

    const keyTypes = {
        BeefyKey: '[u8; 33]',
        Keys: 'SessionKeys4',
        SessionKeys1: '(AccountId)',
        SessionKeys2: '(AccountId, AccountId)',
        SessionKeys3: '(AccountId, AccountId, AccountId)',
        SessionKeys4: '(AccountId, AccountId, AccountId, AccountId)',
        SessionKeys5: '(AccountId, AccountId, AccountId, AccountId, AccountId)',
        SessionKeys6: '(AccountId, AccountId, AccountId, AccountId, AccountId, AccountId)',
        SessionKeys6B: '(AccountId, AccountId, AccountId, AccountId, AccountId, BeefyKey)',
        SessionKeys7: '(AccountId, AccountId, AccountId, AccountId, AccountId, AccountId, AccountId)',
        SessionKeys7B: '(AccountId, AccountId, AccountId, AccountId, AccountId, AccountId, BeefyKey)',
        SessionKeys8: '(AccountId, AccountId, AccountId, AccountId, AccountId, AccountId, AccountId, AccountId)',
        SessionKeys8B: '(AccountId, AccountId, AccountId, AccountId, AccountId, AccountId, AccountId, BeefyKey)',
        SessionKeys9: '(AccountId, AccountId, AccountId, AccountId, AccountId, AccountId, AccountId, AccountId, AccountId)',
        SessionKeys9B: '(AccountId, AccountId, AccountId, AccountId, AccountId, AccountId, AccountId, AccountId, BeefyKey)',
        SessionKeys10: '(AccountId, AccountId, AccountId, AccountId, AccountId, AccountId, AccountId, AccountId, AccountId, AccountId)',
        SessionKeys10B: '(AccountId, AccountId, AccountId, AccountId, AccountId, AccountId, AccountId, AccountId, AccountId, BeefyKey)'
    };
    const definitions$C = {
        rpc: {},
        runtime: runtime$g,
        types: {
            ...keyTypes,
            FullIdentification: 'Exposure',
            IdentificationTuple: '(ValidatorId, FullIdentification)',
            MembershipProof: {
                session: 'SessionIndex',
                trieNodes: 'Vec<Bytes>',
                validatorCount: 'ValidatorCount'
            },
            SessionIndex: 'u32',
            ValidatorCount: 'u32'
        }
    };

    const definitions$B = {
        rpc: {},
        types: {
            Bid: {
                who: 'AccountId',
                kind: 'BidKind',
                value: 'Balance'
            },
            BidKind: {
                _enum: {
                    Deposit: 'Balance',
                    Vouch: '(AccountId, Balance)'
                }
            },
            SocietyJudgement: {
                _enum: ['Rebid', 'Reject', 'Approve']
            },
            SocietyVote: {
                _enum: ['Skeptic', 'Reject', 'Approve']
            },
            StrikeCount: 'u32',
            VouchingStatus: {
                _enum: ['Vouching', 'Banned']
            }
        }
    };

    const runtime$f = {
        StakingApi: [
            {
                methods: {
                    nominations_quota: {
                        description: 'Returns the nominations quota for a nominator with a given balance.',
                        params: [
                            {
                                name: 'balance',
                                type: 'Balance'
                            }
                        ],
                        type: 'u32'
                    }
                },
                version: 1
            }
        ]
    };

    const deprecated = {
        Points: 'u32',
        EraPoints: {
            total: 'Points',
            individual: 'Vec<Points>'
        }
    };
    const phragmen = {
        CompactAssignments: 'CompactAssignmentsWith16',
        CompactAssignmentsWith16: {
            votes1: 'Vec<(NominatorIndexCompact, ValidatorIndexCompact)>',
            votes2: 'Vec<(NominatorIndexCompact, CompactScoreCompact, ValidatorIndexCompact)>',
            votes3: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 2], ValidatorIndexCompact)>',
            votes4: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 3], ValidatorIndexCompact)>',
            votes5: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 4], ValidatorIndexCompact)>',
            votes6: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 5], ValidatorIndexCompact)>',
            votes7: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 6], ValidatorIndexCompact)>',
            votes8: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 7], ValidatorIndexCompact)>',
            votes9: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 8], ValidatorIndexCompact)>',
            votes10: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 9], ValidatorIndexCompact)>',
            votes11: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 10], ValidatorIndexCompact)>',
            votes12: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 11], ValidatorIndexCompact)>',
            votes13: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 12], ValidatorIndexCompact)>',
            votes14: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 13], ValidatorIndexCompact)>',
            votes15: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 14], ValidatorIndexCompact)>',
            votes16: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 15], ValidatorIndexCompact)>'
        },
        CompactAssignmentsWith24: {
            votes1: 'Vec<(NominatorIndexCompact, ValidatorIndexCompact)>',
            votes2: 'Vec<(NominatorIndexCompact, CompactScoreCompact, ValidatorIndexCompact)>',
            votes3: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 2], ValidatorIndexCompact)>',
            votes4: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 3], ValidatorIndexCompact)>',
            votes5: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 4], ValidatorIndexCompact)>',
            votes6: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 5], ValidatorIndexCompact)>',
            votes7: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 6], ValidatorIndexCompact)>',
            votes8: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 7], ValidatorIndexCompact)>',
            votes9: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 8], ValidatorIndexCompact)>',
            votes10: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 9], ValidatorIndexCompact)>',
            votes11: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 10], ValidatorIndexCompact)>',
            votes12: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 11], ValidatorIndexCompact)>',
            votes13: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 12], ValidatorIndexCompact)>',
            votes14: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 13], ValidatorIndexCompact)>',
            votes15: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 14], ValidatorIndexCompact)>',
            votes16: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 15], ValidatorIndexCompact)>',
            votes17: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 16], ValidatorIndexCompact)>',
            votes18: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 17], ValidatorIndexCompact)>',
            votes19: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 18], ValidatorIndexCompact)>',
            votes20: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 19], ValidatorIndexCompact)>',
            votes21: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 20], ValidatorIndexCompact)>',
            votes22: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 21], ValidatorIndexCompact)>',
            votes23: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 22], ValidatorIndexCompact)>',
            votes24: 'Vec<(NominatorIndexCompact, [CompactScoreCompact; 23], ValidatorIndexCompact)>'
        },
        CompactAssignmentsTo265: 'CompactAssignmentsWith16',
        CompactAssignmentsTo257: {
            votes1: 'Vec<(NominatorIndex, [CompactScore; 0], ValidatorIndex)>',
            votes2: 'Vec<(NominatorIndex, [CompactScore; 1], ValidatorIndex)>',
            votes3: 'Vec<(NominatorIndex, [CompactScore; 2], ValidatorIndex)>',
            votes4: 'Vec<(NominatorIndex, [CompactScore; 3], ValidatorIndex)>',
            votes5: 'Vec<(NominatorIndex, [CompactScore; 4], ValidatorIndex)>',
            votes6: 'Vec<(NominatorIndex, [CompactScore; 5], ValidatorIndex)>',
            votes7: 'Vec<(NominatorIndex, [CompactScore; 6], ValidatorIndex)>',
            votes8: 'Vec<(NominatorIndex, [CompactScore; 7], ValidatorIndex)>',
            votes9: 'Vec<(NominatorIndex, [CompactScore; 8], ValidatorIndex)>',
            votes10: 'Vec<(NominatorIndex, [CompactScore; 9], ValidatorIndex)>',
            votes11: 'Vec<(NominatorIndex, [CompactScore; 10], ValidatorIndex)>',
            votes12: 'Vec<(NominatorIndex, [CompactScore; 11], ValidatorIndex)>',
            votes13: 'Vec<(NominatorIndex, [CompactScore; 12], ValidatorIndex)>',
            votes14: 'Vec<(NominatorIndex, [CompactScore; 13], ValidatorIndex)>',
            votes15: 'Vec<(NominatorIndex, [CompactScore; 14], ValidatorIndex)>',
            votes16: 'Vec<(NominatorIndex, [CompactScore; 15], ValidatorIndex)>'
        },
        CompactScore: '(ValidatorIndex, OffchainAccuracy)',
        CompactScoreCompact: '(ValidatorIndexCompact, OffchainAccuracyCompact)',
        ElectionCompute: {
            _enum: ['OnChain', 'Signed', 'Unsigned']
        },
        ElectionPhase: {
            _enum: {
                Off: null,
                Signed: null,
                Unsigned: '(bool, BlockNumber)',
                Emergency: null
            }
        },
        ElectionResult: {
            compute: 'ElectionCompute',
            slotStake: 'Balance',
            electedStashes: 'Vec<AccountId>',
            exposures: 'Vec<(AccountId, Exposure)>'
        },
        ElectionResultToSpec10: {
            electedStashes: 'Vec<AccountId>',
            exposures: 'Vec<(AccountId, Exposure<AccountId, Balance>)>',
            compute: 'ElectionCompute'
        },
        ElectionScore: '[u128; 3]',
        ElectionSize: {
            validators: 'Compact<ValidatorIndex>',
            nominators: 'Compact<NominatorIndex>'
        },
        ElectionStatus: {
            _enum: {
                Close: 'Null',
                Open: 'BlockNumber'
            }
        },
        ExtendedBalance: 'u128',
        RawSolution: 'RawSolutionWith16',
        RawSolutionWith16: {
            compact: 'CompactAssignmentsWith16',
            score: 'ElectionScore',
            round: 'u32'
        },
        RawSolutionWith24: {
            compact: 'CompactAssignmentsWith24',
            score: 'ElectionScore',
            round: 'u32'
        },
        RawSolutionTo265: 'RawSolutionWith16',
        ReadySolution: {
            supports: 'SolutionSupports',
            score: 'ElectionScore',
            compute: 'ElectionCompute'
        },
        RoundSnapshot: {
            voters: 'Vec<(AccountId, VoteWeight, Vec<AccountId>)>',
            targets: 'Vec<AccountId>'
        },
        SeatHolder: {
            who: 'AccountId',
            stake: 'Balance',
            deposit: 'Balance'
        },
        SignedSubmission: {
            _fallback: 'SignedSubmissionTo276',
            who: 'AccountId',
            deposit: 'Balance',
            solution: 'RawSolution',
            reward: 'Balance'
        },
        SignedSubmissionTo276: {
            who: 'AccountId',
            deposit: 'Balance',
            solution: 'RawSolution'
        },
        SignedSubmissionOf: 'SignedSubmission',
        SolutionOrSnapshotSize: {
            voters: 'Compact<u32>',
            targets: 'Compact<u32>'
        },
        SolutionSupport: {
            total: 'ExtendedBalance',
            voters: 'Vec<(AccountId, ExtendedBalance)>'
        },
        SolutionSupports: 'Vec<(AccountId, SolutionSupport)>',
        Supports: 'SolutionSupports',
        SubmissionIndicesOf: 'BTreeMap<ElectionScore, u32>',
        Voter: {
            votes: 'Vec<AccountId>',
            stake: 'Balance',
            deposit: 'Balance'
        },
        VoteWeight: 'u64'
    };
    const definitions$A = {
        rpc: {},
        runtime: runtime$f,
        types: {
            ...deprecated,
            ...phragmen,
            ActiveEraInfo: {
                index: 'EraIndex',
                start: 'Option<Moment>'
            },
            EraIndex: 'u32',
            EraRewardPoints: {
                total: 'RewardPoint',
                individual: 'BTreeMap<AccountId, RewardPoint>'
            },
            EraRewards: {
                total: 'u32',
                rewards: 'Vec<u32>'
            },
            Exposure: {
                total: 'Compact<Balance>',
                own: 'Compact<Balance>',
                others: 'Vec<IndividualExposure>'
            },
            Forcing: {
                _enum: [
                    'NotForcing',
                    'ForceNew',
                    'ForceNone',
                    'ForceAlways'
                ]
            },
            IndividualExposure: {
                who: 'AccountId',
                value: 'Compact<Balance>'
            },
            KeyType: 'AccountId',
            MomentOf: 'Moment',
            Nominations: {
                targets: 'Vec<AccountId>',
                submittedIn: 'EraIndex',
                suppressed: 'bool'
            },
            NominatorIndex: 'u32',
            NominatorIndexCompact: 'Compact<NominatorIndex>',
            OffchainAccuracy: 'PerU16',
            OffchainAccuracyCompact: 'Compact<OffchainAccuracy>',
            PhragmenScore: '[u128; 3]',
            Points: 'u32',
            RewardDestination: {
                _enum: {
                    Staked: 'Null',
                    Stash: 'Null',
                    Controller: 'Null',
                    Account: 'AccountId',
                    None: 'Null'
                }
            },
            RewardPoint: 'u32',
            SlashJournalEntry: {
                who: 'AccountId',
                amount: 'Balance',
                ownSlash: 'Balance'
            },
            SlashingSpansTo204: {
                spanIndex: 'SpanIndex',
                lastStart: 'EraIndex',
                prior: 'Vec<EraIndex>'
            },
            SlashingSpans: {
                spanIndex: 'SpanIndex',
                lastStart: 'EraIndex',
                lastNonzeroSlash: 'EraIndex',
                prior: 'Vec<EraIndex>'
            },
            SpanIndex: 'u32',
            SpanRecord: {
                slashed: 'Balance',
                paidOut: 'Balance'
            },
            StakingLedgerTo223: {
                stash: 'AccountId',
                total: 'Compact<Balance>',
                active: 'Compact<Balance>',
                unlocking: 'Vec<UnlockChunk>'
            },
            StakingLedgerTo240: {
                _fallback: 'StakingLedgerTo223',
                stash: 'AccountId',
                total: 'Compact<Balance>',
                active: 'Compact<Balance>',
                unlocking: 'Vec<UnlockChunk>',
                lastReward: 'Option<EraIndex>'
            },
            StakingLedger: {
                stash: 'AccountId',
                total: 'Compact<Balance>',
                active: 'Compact<Balance>',
                unlocking: 'Vec<UnlockChunk>',
                claimedRewards: 'Vec<EraIndex>'
            },
            UnappliedSlashOther: '(AccountId, Balance)',
            UnappliedSlash: {
                validator: 'AccountId',
                own: 'Balance',
                others: 'Vec<UnappliedSlashOther>',
                reporters: 'Vec<AccountId>',
                payout: 'Balance'
            },
            UnlockChunk: {
                value: 'Compact<Balance>',
                era: 'Compact<BlockNumber>'
            },
            ValidatorIndex: 'u16',
            ValidatorIndexCompact: 'Compact<ValidatorIndex>',
            ValidatorPrefs: 'ValidatorPrefsWithBlocked',
            ValidatorPrefsWithCommission: {
                commission: 'Compact<Perbill>'
            },
            ValidatorPrefsWithBlocked: {
                commission: 'Compact<Perbill>',
                blocked: 'bool'
            },
            ValidatorPrefsTo196: {
                validatorPayment: 'Compact<Balance>'
            },
            ValidatorPrefsTo145: {
                unstakeThreshold: 'Compact<u32>',
                validatorPayment: 'Compact<Balance>'
            }
        }
    };

    const runtime$e = {
        ValidateStatement: [
            {
                methods: {
                    valdate_statement: {
                        description: 'Validate the statement.',
                        params: [
                            {
                                name: 'source',
                                type: 'StatementStoreStatementSource'
                            },
                            {
                                name: 'statement',
                                type: 'SpStatementStoreStatement'
                            }
                        ],
                        type: 'Result<StatementStoreValidStatement, StatementStoreInvalidStatement>'
                    }
                },
                version: 1
            }
        ]
    };

    const definitions$z = {
        rpc: {},
        runtime: runtime$e,
        types: {
            StatementStoreStatementSource: {
                _enum: ['Chain', 'Network', 'Local']
            },
            StatementStoreValidStatement: {
                maxCount: 'u32',
                maxSize: 'u32'
            },
            StatementStoreInvalidStatement: {
                _enum: ['BadProof', 'NoProof', 'InternalError']
            }
        }
    };

    const definitions$y = {
        rpc: {},
        types: {
            WeightToFeeCoefficient: {
                coeffInteger: 'Balance',
                coeffFrac: 'Perbill',
                negative: 'bool',
                degree: 'u8'
            }
        }
    };

    const rpc$9 = {
        genSyncSpec: {
            description: 'Returns the json-serialized chainspec running the node, with a sync state.',
            endpoint: 'sync_state_genSyncSpec',
            params: [
                {
                    name: 'raw',
                    type: 'bool'
                }
            ],
            type: 'Json'
        }
    };

    const definitions$x = {
        rpc: rpc$9,
        types: {}
    };

    const rpc$8 = {
        accountNextIndex: {
            alias: ['account_nextIndex'],
            description: 'Retrieves the next accountIndex as available on the node',
            params: [
                {
                    name: 'accountId',
                    type: 'AccountId'
                }
            ],
            type: 'Index'
        },
        addLogFilter: {
            description: 'Adds the supplied directives to the current log filter',
            isUnsafe: true,
            params: [
                {
                    name: 'directives',
                    type: 'Text'
                }
            ],
            type: 'Null'
        },
        addReservedPeer: {
            description: 'Adds a reserved peer',
            isUnsafe: true,
            params: [
                {
                    name: 'peer',
                    type: 'Text'
                }
            ],
            type: 'Text'
        },
        chain: {
            description: 'Retrieves the chain',
            params: [],
            type: 'Text'
        },
        chainType: {
            description: 'Retrieves the chain type',
            params: [],
            type: 'ChainType'
        },
        dryRun: {
            alias: ['system_dryRunAt'],
            description: 'Dry run an extrinsic at a given block',
            isUnsafe: true,
            params: [
                {
                    name: 'extrinsic',
                    type: 'Bytes'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'ApplyExtrinsicResult'
        },
        health: {
            description: 'Return health status of the node',
            noErrorLog: true,
            params: [],
            type: 'Health'
        },
        localListenAddresses: {
            description: 'The addresses include a trailing /p2p/ with the local PeerId, and are thus suitable to be passed to addReservedPeer or as a bootnode address for example',
            params: [],
            type: 'Vec<Text>'
        },
        localPeerId: {
            description: 'Returns the base58-encoded PeerId of the node',
            params: [],
            type: 'Text'
        },
        name: {
            description: 'Retrieves the node name',
            params: [],
            type: 'Text'
        },
        networkState: {
            alias: ['system_unstable_networkState'],
            description: 'Returns current state of the network',
            isUnsafe: true,
            params: [],
            type: 'NetworkState'
        },
        nodeRoles: {
            description: 'Returns the roles the node is running as',
            params: [],
            type: 'Vec<NodeRole>'
        },
        peers: {
            description: 'Returns the currently connected peers',
            isUnsafe: true,
            params: [],
            type: 'Vec<PeerInfo>'
        },
        properties: {
            description: 'Get a custom set of properties as a JSON object, defined in the chain spec',
            params: [],
            type: 'ChainProperties'
        },
        removeReservedPeer: {
            description: 'Remove a reserved peer',
            isUnsafe: true,
            params: [
                {
                    name: 'peerId',
                    type: 'Text'
                }
            ],
            type: 'Text'
        },
        reservedPeers: {
            description: 'Returns the list of reserved peers',
            params: [],
            type: 'Vec<Text>'
        },
        resetLogFilter: {
            description: 'Resets the log filter to Substrate defaults',
            isUnsafe: true,
            params: [],
            type: 'Null'
        },
        syncState: {
            description: 'Returns the state of the syncing of the node',
            params: [],
            type: 'SyncState'
        },
        version: {
            description: 'Retrieves the version of the node',
            params: [],
            type: 'Text'
        }
    };

    const runtime$d = {
        AccountNonceApi: [
            {
                methods: {
                    account_nonce: {
                        description: 'The API to query account nonce (aka transaction index)',
                        params: [
                            {
                                name: 'accountId',
                                type: 'AccountId'
                            }
                        ],
                        type: 'Index'
                    }
                },
                version: 1
            }
        ]
    };

    const definitions$w = {
        rpc: rpc$8,
        runtime: runtime$d,
        types: {
            AccountInfo: 'AccountInfoWithTripleRefCount',
            AccountInfoWithRefCountU8: {
                nonce: 'Index',
                refcount: 'u8',
                data: 'AccountData'
            },
            AccountInfoWithRefCount: {
                _fallback: 'AccountInfoWithRefCountU8',
                nonce: 'Index',
                refcount: 'RefCount',
                data: 'AccountData'
            },
            AccountInfoWithDualRefCount: {
                _fallback: 'AccountInfoWithRefCount',
                nonce: 'Index',
                consumers: 'RefCount',
                providers: 'RefCount',
                data: 'AccountData'
            },
            AccountInfoWithProviders: 'AccountInfoWithDualRefCount',
            AccountInfoWithTripleRefCount: {
                _fallback: 'AccountInfoWithDualRefCount',
                nonce: 'Index',
                consumers: 'RefCount',
                providers: 'RefCount',
                sufficients: 'RefCount',
                data: 'AccountData'
            },
            ApplyExtrinsicResult: 'Result<DispatchOutcome, TransactionValidityError>',
            ApplyExtrinsicResultPre6: 'Result<DispatchOutcomePre6, TransactionValidityError>',
            ArithmeticError: {
                _enum: [
                    'Underflow',
                    'Overflow',
                    'DivisionByZero'
                ]
            },
            BlockLength: {
                max: 'PerDispatchClassU32'
            },
            BlockWeights: {
                baseBlock: 'Weight',
                maxBlock: 'Weight',
                perClass: 'PerDispatchClassWeightsPerClass'
            },
            ChainProperties: 'GenericChainProperties',
            ChainType: {
                _enum: {
                    Development: 'Null',
                    Local: 'Null',
                    Live: 'Null',
                    Custom: 'Text'
                }
            },
            ConsumedWeight: 'PerDispatchClassWeight',
            DigestOf: 'Digest',
            DispatchClass: {
                _enum: ['Normal', 'Operational', 'Mandatory']
            },
            DispatchError: {
                _enum: {
                    Other: 'Null',
                    CannotLookup: 'Null',
                    BadOrigin: 'Null',
                    Module: 'DispatchErrorModule',
                    ConsumerRemaining: 'Null',
                    NoProviders: 'Null',
                    TooManyConsumers: 'Null',
                    Token: 'TokenError',
                    Arithmetic: 'ArithmeticError',
                    Transactional: 'TransactionalError',
                    Exhausted: 'Null',
                    Corruption: 'Null',
                    Unavailable: 'Null'
                }
            },
            DispatchErrorPre6: {
                _enum: {
                    Other: 'Null',
                    CannotLookup: 'Null',
                    BadOrigin: 'Null',
                    Module: 'DispatchErrorModulePre6',
                    ConsumerRemaining: 'Null',
                    NoProviders: 'Null',
                    TooManyConsumers: 'Null',
                    Token: 'TokenError',
                    Arithmetic: 'ArithmeticError',
                    Transactional: 'TransactionalError'
                }
            },
            DispatchErrorPre6First: {
                _enum: {
                    Other: 'Null',
                    CannotLookup: 'Null',
                    BadOrigin: 'Null',
                    Module: 'DispatchErrorModulePre6',
                    ConsumerRemaining: 'Null',
                    NoProviders: 'Null',
                    Token: 'TokenError',
                    Arithmetic: 'ArithmeticError',
                    Transactional: 'TransactionalError'
                }
            },
            DispatchErrorModuleU8: {
                index: 'u8',
                error: 'u8'
            },
            DispatchErrorModuleU8a: {
                index: 'u8',
                error: '[u8; 4]'
            },
            DispatchErrorModule: 'DispatchErrorModuleU8a',
            DispatchErrorModulePre6: 'DispatchErrorModuleU8',
            DispatchErrorTo198: {
                module: 'Option<u8>',
                error: 'u8'
            },
            DispatchInfo: {
                weight: 'Weight',
                class: 'DispatchClass',
                paysFee: 'Pays'
            },
            DispatchInfoTo190: {
                weight: 'Weight',
                class: 'DispatchClass'
            },
            DispatchInfoTo244: {
                weight: 'Weight',
                class: 'DispatchClass',
                paysFee: 'bool'
            },
            DispatchOutcome: 'Result<(), DispatchError>',
            DispatchOutcomePre6: 'Result<(), DispatchErrorPre6>',
            DispatchResult: 'Result<(), DispatchError>',
            DispatchResultOf: 'DispatchResult',
            DispatchResultTo198: 'Result<(), Text>',
            Event: 'GenericEvent',
            EventId: '[u8; 2]',
            EventIndex: 'u32',
            EventRecord: {
                phase: 'Phase',
                event: 'Event',
                topics: 'Vec<Hash>'
            },
            Health: {
                peers: 'u64',
                isSyncing: 'bool',
                shouldHavePeers: 'bool'
            },
            InvalidTransaction: {
                _enum: {
                    Call: 'Null',
                    Payment: 'Null',
                    Future: 'Null',
                    Stale: 'Null',
                    BadProof: 'Null',
                    AncientBirthBlock: 'Null',
                    ExhaustsResources: 'Null',
                    Custom: 'u8',
                    BadMandatory: 'Null',
                    MandatoryDispatch: 'Null',
                    BadSigner: 'Null'
                }
            },
            Key: 'Bytes',
            LastRuntimeUpgradeInfo: {
                specVersion: 'Compact<u32>',
                specName: 'Text'
            },
            NetworkState: {
                peerId: 'Text',
                listenedAddresses: 'Vec<Text>',
                externalAddresses: 'Vec<Text>',
                connectedPeers: 'HashMap<Text, Peer>',
                notConnectedPeers: 'HashMap<Text, NotConnectedPeer>',
                averageDownloadPerSec: 'u64',
                averageUploadPerSec: 'u64',
                peerset: 'NetworkStatePeerset'
            },
            NetworkStatePeerset: {
                messageQueue: 'u64',
                nodes: 'HashMap<Text, NetworkStatePeersetInfo>'
            },
            NetworkStatePeersetInfo: {
                connected: 'bool',
                reputation: 'i32'
            },
            NodeRole: {
                _enum: {
                    Full: 'Null',
                    LightClient: 'Null',
                    Authority: 'Null',
                    UnknownRole: 'u8'
                }
            },
            NotConnectedPeer: {
                knownAddresses: 'Vec<Text>',
                latestPingTime: 'Option<PeerPing>',
                versionString: 'Option<Text>'
            },
            Peer: {
                enabled: 'bool',
                endpoint: 'PeerEndpoint',
                knownAddresses: 'Vec<Text>',
                latestPingTime: 'PeerPing',
                open: 'bool',
                versionString: 'Text'
            },
            PeerEndpoint: {
                listening: 'PeerEndpointAddr'
            },
            PeerEndpointAddr: {
                _alias: {
                    localAddr: 'local_addr',
                    sendBackAddr: 'send_back_addr'
                },
                localAddr: 'Text',
                sendBackAddr: 'Text'
            },
            PeerPing: {
                nanos: 'u64',
                secs: 'u64'
            },
            PeerInfo: {
                peerId: 'Text',
                roles: 'Text',
                protocolVersion: 'u32',
                bestHash: 'Hash',
                bestNumber: 'BlockNumber'
            },
            PerDispatchClassU32: {
                normal: 'u32',
                operational: 'u32',
                mandatory: 'u32'
            },
            PerDispatchClassWeight: {
                normal: 'Weight',
                operational: 'Weight',
                mandatory: 'Weight'
            },
            PerDispatchClassWeightsPerClass: {
                normal: 'WeightPerClass',
                operational: 'WeightPerClass',
                mandatory: 'WeightPerClass'
            },
            Phase: {
                _enum: {
                    ApplyExtrinsic: 'u32',
                    Finalization: 'Null',
                    Initialization: 'Null'
                }
            },
            RawOrigin: {
                _enum: {
                    Root: 'Null',
                    Signed: 'AccountId',
                    None: 'Null'
                }
            },
            RefCount: 'u32',
            RefCountTo259: 'u8',
            SyncState: {
                startingBlock: 'BlockNumber',
                currentBlock: 'BlockNumber',
                highestBlock: 'Option<BlockNumber>'
            },
            SystemOrigin: 'RawOrigin',
            TokenError: {
                _enum: [
                    'NoFunds',
                    'WouldDie',
                    'BelowMinimum',
                    'CannotCreate',
                    'UnknownAsset',
                    'Frozen',
                    'Unsupported',
                    'Underflow',
                    'Overflow'
                ]
            },
            TransactionValidityError: {
                _enum: {
                    Invalid: 'InvalidTransaction',
                    Unknown: 'UnknownTransaction'
                }
            },
            TransactionalError: {
                _enum: [
                    'LimitReached',
                    'NoLayer'
                ]
            },
            UnknownTransaction: {
                _enum: {
                    CannotLookup: 'Null',
                    NoUnsignedValidator: 'Null',
                    Custom: 'u8'
                }
            },
            WeightPerClass: {
                baseExtrinsic: 'Weight',
                maxExtrinsic: 'Option<Weight>',
                maxTotal: 'Option<Weight>',
                reserved: 'Option<Weight>'
            }
        }
    };

    const definitions$v = {
        rpc: {},
        types: {
            Bounty: {
                proposer: 'AccountId',
                value: 'Balance',
                fee: 'Balance',
                curatorDeposit: 'Balance',
                bond: 'Balance',
                status: 'BountyStatus'
            },
            BountyIndex: 'u32',
            BountyStatus: {
                _enum: {
                    Proposed: 'Null',
                    Approved: 'Null',
                    Funded: 'Null',
                    CuratorProposed: 'BountyStatusCuratorProposed',
                    Active: 'BountyStatusActive',
                    PendingPayout: 'BountyStatusPendingPayout'
                }
            },
            BountyStatusActive: {
                curator: 'AccountId',
                updateDue: 'BlockNumber'
            },
            BountyStatusCuratorProposed: {
                curator: 'AccountId'
            },
            BountyStatusPendingPayout: {
                curator: 'AccountId',
                beneficiary: 'AccountId',
                unlockAt: 'BlockNumber'
            },
            OpenTip: {
                reason: 'Hash',
                who: 'AccountId',
                finder: 'AccountId',
                deposit: 'Balance',
                closes: 'Option<BlockNumber>',
                tips: 'Vec<OpenTipTip>',
                findersFee: 'bool'
            },
            OpenTipTo225: {
                reason: 'Hash',
                who: 'AccountId',
                finder: 'Option<OpenTipFinderTo225>',
                closes: 'Option<BlockNumber>',
                tips: 'Vec<OpenTipTip>'
            },
            OpenTipFinderTo225: '(AccountId, Balance)',
            OpenTipTip: '(AccountId, Balance)',
            TreasuryProposal: {
                proposer: 'AccountId',
                value: 'Balance',
                beneficiary: 'AccountId',
                bond: 'Balance'
            }
        }
    };

    const definitions$u = {
        rpc: {},
        types: {
            Multiplier: 'Fixed128'
        }
    };

    const runtime$c = {
        TaggedTransactionQueue: [
            {
                methods: {
                    validate_transaction: {
                        description: 'Validate the transaction.',
                        params: [
                            {
                                name: 'source',
                                type: 'TransactionSource'
                            },
                            {
                                name: 'tx',
                                type: 'Extrinsic'
                            },
                            {
                                name: 'blockHash',
                                type: 'BlockHash'
                            }
                        ],
                        type: 'TransactionValidity'
                    }
                },
                version: 3
            },
            {
                methods: {
                    validate_transaction: {
                        description: 'Validate the transaction.',
                        params: [
                            {
                                name: 'source',
                                type: 'TransactionSource'
                            },
                            {
                                name: 'tx',
                                type: 'Extrinsic'
                            }
                        ],
                        type: 'TransactionValidity'
                    }
                },
                version: 2
            },
            {
                methods: {
                    validate_transaction: {
                        description: 'Validate the transaction.',
                        params: [
                            {
                                name: 'tx',
                                type: 'Extrinsic'
                            }
                        ],
                        type: 'TransactionValidity'
                    }
                },
                version: 1
            }
        ]
    };

    const definitions$t = {
        rpc: {},
        runtime: runtime$c,
        types: {
            TransactionSource: {
                _enum: ['InBlock', 'Local', 'External']
            },
            TransactionValidity: 'Result<ValidTransaction, TransactionValidityError>',
            ValidTransaction: {
                priority: 'TransactionPriority',
                requires: 'Vec<TransactionTag>',
                provides: 'Vec<TransactionTag>',
                longevity: 'TransactionLongevity',
                propagate: 'bool'
            }
        }
    };

    const definitions$s = {
        rpc: {},
        types: {
            ClassId: 'u32',
            InstanceId: 'u32',
            DepositBalance: 'Balance',
            DepositBalanceOf: 'Balance',
            ClassDetails: {
                owner: 'AccountId',
                issuer: 'AccountId',
                admin: 'AccountId',
                freezer: 'AccountId',
                totalDeposit: 'DepositBalance',
                freeHolding: 'bool',
                instances: 'u32',
                instanceMetadatas: 'u32',
                attributes: 'u32',
                isFrozen: 'bool'
            },
            DestroyWitness: {
                instances: 'Compact<u32>',
                instanceMetadatas: 'Compact<u32>',
                attributes: 'Compact<u32>'
            },
            InstanceDetails: {
                owner: 'AccountId',
                approved: 'Option<AccountId>',
                isFrozen: 'bool',
                deposit: 'DepositBalance'
            },
            ClassMetadata: {
                deposit: 'DepositBalance',
                data: 'Vec<u8>',
                isFrozen: 'bool'
            },
            InstanceMetadata: {
                deposit: 'DepositBalance',
                data: 'Vec<u8>',
                isFrozen: 'bool'
            }
        }
    };

    const definitions$r = {
        rpc: {},
        types: {
            Multisig: {
                when: 'Timepoint',
                deposit: 'Balance',
                depositor: 'AccountId',
                approvals: 'Vec<AccountId>'
            },
            Timepoint: {
                height: 'BlockNumber',
                index: 'u32'
            }
        }
    };

    const definitions$q = {
        rpc: {},
        types: {
            VestingInfo: {
                locked: 'Balance',
                perBlock: 'Balance',
                startingBlock: 'BlockNumber'
            }
        }
    };

    const definitions$p = {
        rpc: {},
        types: {
            BlockAttestations: {
                receipt: 'CandidateReceipt',
                valid: 'Vec<AccountId>',
                invalid: 'Vec<AccountId>'
            },
            IncludedBlocks: {
                actualNumber: 'BlockNumber',
                session: 'SessionIndex',
                randomSeed: 'H256',
                activeParachains: 'Vec<ParaId>',
                paraBlocks: 'Vec<Hash>'
            },
            MoreAttestations: {}
        }
    };

    const definitions$o = {
        rpc: {},
        types: {
            BridgedBlockHash: 'H256',
            BridgedBlockNumber: 'BlockNumber',
            BridgedHeader: 'Header',
            BridgeMessageId: '(LaneId, MessageNonce)',
            CallOrigin: {
                _enum: {
                    SourceRoot: 'Null',
                    TargetAccount: '(AccountId, MultiSigner, MultiSignature)',
                    SourceAccount: 'AccountId'
                }
            },
            ChainId: '[u8; 4]',
            DeliveredMessages: {
                begin: 'MessageNonce',
                end: 'MessageNonce',
                dispatchResults: 'BitVec'
            },
            DispatchFeePayment: {
                _enum: ['AtSourceChain', 'AtTargetChain']
            },
            InboundLaneData: {
                relayers: 'Vec<UnrewardedRelayer>',
                lastConfirmedNonce: 'MessageNonce'
            },
            InboundRelayer: 'AccountId',
            InitializationData: {
                header: 'Header',
                authorityList: 'AuthorityList',
                setId: 'SetId',
                isHalted: 'bool'
            },
            LaneId: '[u8; 4]',
            MessageData: {
                payload: 'Bytes',
                fee: 'Balance'
            },
            MessagesDeliveryProofOf: {
                bridgedHeaderHash: 'BlockHash',
                storageProof: 'Vec<Bytes>',
                lane: 'LaneId'
            },
            MessageKey: {
                laneId: 'LaneId',
                nonce: 'MessageNonce'
            },
            MessageNonce: 'u64',
            MessagesProofOf: {
                bridgedHeaderHash: 'BridgedBlockHash',
                storageProof: 'Vec<Bytes>',
                lane: 'LaneId',
                noncesStart: 'MessageNonce',
                noncesEnd: 'MessageNonce'
            },
            OperatingMode: {
                _enum: ['Normal', 'RejectingOutboundMessages', 'Halted']
            },
            OutboundLaneData: {
                oldestUnprunedNonce: 'MessageNonce',
                latestReceivedNonce: 'MessageNonce',
                latestGeneratedNonce: 'MessageNonce'
            },
            OutboundMessageFee: 'Balance',
            OutboundPayload: {
                specVersion: 'u32',
                weight: 'Weight',
                origin: 'CallOrigin',
                dispatchFeePayment: 'DispatchFeePayment',
                call: 'Bytes'
            },
            Parameter: 'Null',
            RelayerId: 'AccountId',
            UnrewardedRelayer: {
                relayer: 'RelayerId',
                messages: 'DeliveredMessages'
            },
            UnrewardedRelayersState: {
                unrewardedRelayer_Entries: 'MessageNonce',
                messagesInOldestEntry: 'MessageNonce',
                totalMessages: 'MessageNonce'
            }
        }
    };

    const definitions$n = {
        rpc: {},
        types: {
            StatementKind: {
                _enum: ['Regular', 'Saft']
            }
        }
    };

    const definitions$m = {
        rpc: {},
        types: {
            FundIndex: 'u32',
            LastContribution: {
                _enum: {
                    Never: 'Null',
                    PreEnding: 'u32',
                    Ending: 'BlockNumber'
                }
            },
            FundInfo: {
                depositor: 'AccountId',
                verifier: 'Option<MultiSigner>',
                deposit: 'Balance',
                raised: 'Balance',
                end: 'BlockNumber',
                cap: 'Balance',
                lastContribution: 'LastContribution',
                firstPeriod: 'LeasePeriod',
                lastPeriod: 'LeasePeriod',
                trieIndex: 'TrieIndex'
            },
            TrieIndex: 'u32'
        }
    };

    const runtime$b = {
        AuraUnincludedSegmentApi: [
            {
                methods: {
                    can_build_upon: {
                        description: 'Whether it is legal to extend the chain',
                        params: [
                            {
                                name: 'includedHash',
                                type: 'BlockHash'
                            },
                            {
                                name: 'slot',
                                type: 'Slot'
                            }
                        ],
                        type: 'bool'
                    }
                },
                version: 1
            }
        ],
        CollectCollationInfo: [
            {
                methods: {
                    collect_collation_info: {
                        description: 'Collect information about a collation.',
                        params: [
                            {
                                name: 'header',
                                type: 'Header'
                            }
                        ],
                        type: 'CollationInfo'
                    }
                },
                version: 2
            },
            {
                methods: {
                    collect_collation_info: {
                        description: 'Collect information about a collation.',
                        params: [],
                        type: 'CollationInfoV1'
                    }
                },
                version: 1
            }
        ]
    };

    const dmpQueue = {
        CollationInfo: {
            upwardMessages: 'Vec<UpwardMessage>',
            horizontalMessages: 'Vec<OutboundHrmpMessage>',
            newValidationCode: 'Option<ValidationCode>',
            processedDownwardMessages: 'u32',
            hrmpWatermark: 'RelayBlockNumber',
            headData: 'HeadData'
        },
        CollationInfoV1: {
            upwardMessages: 'Vec<UpwardMessage>',
            horizontalMessages: 'Vec<OutboundHrmpMessage>',
            newValidationCode: 'Option<ValidationCode>',
            processedDownwardMessages: 'u32',
            hrmpWatermark: 'RelayBlockNumber'
        },
        ConfigData: {
            maxIndividual: 'Weight'
        },
        MessageId: '[u8; 32]',
        OverweightIndex: 'u64',
        PageCounter: 'u32',
        PageIndexData: {
            beginUsed: 'PageCounter',
            endUsed: 'PageCounter',
            overweightCount: 'OverweightIndex'
        }
    };
    const definitions$l = {
        rpc: {},
        runtime: runtime$b,
        types: dmpQueue
    };

    const finalityV1 = {
        methods: {
            best_finalized: {
                description: 'Returns number and hash of the best finalized header known to the bridge module.',
                params: [],
                type: '(BlockNumber, Hash)'
            }
        },
        version: 1
    };
    const runtime$a = {
        KusamaFinalityApi: [finalityV1],
        PolkadotFinalityApi: [finalityV1],
        RococoFinalityApi: [finalityV1],
        WestendFinalityApi: [finalityV1]
    };

    const definitions$k = {
        rpc: {},
        runtime: runtime$a,
        types: {}
    };

    const hrmpTypes = {
        HrmpChannel: {
            maxCapacity: 'u32',
            maxTotalSize: 'u32',
            maxMessageSize: 'u32',
            msgCount: 'u32',
            totalSize: 'u32',
            mqcHead: 'Option<Hash>',
            senderDeposit: 'Balance',
            recipientDeposit: 'Balance'
        },
        HrmpChannelId: {
            sender: 'u32',
            receiver: 'u32'
        },
        HrmpOpenChannelRequest: {
            confirmed: 'bool',
            age: 'SessionIndex',
            senderDeposit: 'Balance',
            maxMessageSize: 'u32',
            maxCapacity: 'u32',
            maxTotalSize: 'u32'
        }
    };

    const PH_V1_TO_V2 = {
        assumed_validation_data: {
            description: 'Returns the persisted validation data for the given `ParaId` along with the corresponding validation code hash.',
            params: [
                {
                    name: 'paraId',
                    type: 'ParaId'
                },
                {
                    name: 'hash',
                    type: 'Hash'
                }
            ],
            type: 'Option<(PersistedValidationData, ValidationCodeHash)>'
        },
        availability_cores: {
            description: 'Yields information on all availability cores as relevant to the child block.',
            params: [],
            type: 'Vec<CoreState>'
        },
        candidate_events: {
            description: 'Get a vector of events concerning candidates that occurred within a block.',
            params: [],
            type: 'Vec<CandidateEvent>'
        },
        candidate_pending_availability: {
            description: 'Get the receipt of a candidate pending availability.',
            params: [
                {
                    name: 'paraId',
                    type: 'ParaId'
                }
            ],
            type: 'Option<CommittedCandidateReceipt>'
        },
        check_validation_outputs: {
            description: 'Checks if the given validation outputs pass the acceptance criteria.',
            params: [
                {
                    name: 'paraId',
                    type: 'ParaId'
                },
                {
                    name: 'outputs',
                    type: 'CandidateCommitments'
                }
            ],
            type: 'bool'
        },
        dmq_contents: {
            description: 'Get all the pending inbound messages in the downward message queue for a para.',
            params: [
                {
                    name: 'paraId',
                    type: 'ParaId'
                }
            ],
            type: 'Vec<InboundDownwardMessage>'
        },
        inbound_hrmp_channels_contents: {
            description: 'Get the contents of all channels addressed to the given recipient.',
            params: [
                {
                    name: 'paraId',
                    type: 'ParaId'
                }
            ],
            type: 'Vec<InboundHrmpMessage>'
        },
        on_chain_votes: {
            description: 'Scrape dispute relevant from on-chain, backing votes and resolved disputes.',
            params: [],
            type: 'Option<ScrapedOnChainVotes>'
        },
        persisted_validation_data: {
            description: 'Yields the persisted validation data for the given `ParaId` along with an assumption that should be used if the para currently occupies a core.',
            params: [
                {
                    name: 'paraId',
                    type: 'ParaId'
                },
                {
                    name: 'assumption',
                    type: 'OccupiedCoreAssumption'
                }
            ],
            type: 'Option<PersistedValidationData>'
        },
        session_index_for_child: {
            description: 'Returns the session index expected at a child of the block.',
            params: [],
            type: 'SessionIndex'
        },
        validation_code: {
            description: 'Fetch the validation code used by a para, making the given `OccupiedCoreAssumption`.',
            params: [
                {
                    name: 'paraId',
                    type: 'ParaId'
                },
                {
                    name: 'assumption',
                    type: 'OccupiedCoreAssumption'
                }
            ],
            type: 'Option<ValidationCode>'
        },
        validation_code_by_hash: {
            description: 'Get the validation code from its hash.',
            params: [
                {
                    name: 'hash',
                    type: 'ValidationCodeHash'
                }
            ],
            type: 'Option<ValidationCode>'
        },
        validator_groups: {
            description: 'Returns the validator groups and rotation info localized based on the hypothetical child of a block whose state  this is invoked on',
            params: [],
            type: '(Vec<Vec<ParaValidatorIndex>>, GroupRotationInfo)'
        },
        validators: {
            description: 'Get the current validators.',
            params: [],
            type: 'Vec<ValidatorId>'
        }
    };
    const PH_V2_TO_V3 = {
        pvfs_require_precheck: {
            description: 'Returns code hashes of PVFs that require pre-checking by validators in the active set.',
            params: [],
            type: 'Vec<ValidationCodeHash>'
        },
        session_info: {
            description: 'Get the session info for the given session, if stored.',
            params: [
                {
                    name: 'index',
                    type: 'SessionIndex'
                }
            ],
            type: 'Option<SessionInfo>'
        },
        submit_pvf_check_statement: {
            description: 'Submits a PVF pre-checking statement into the transaction pool.',
            params: [
                {
                    name: 'stmt',
                    type: 'PvfCheckStatement'
                },
                {
                    name: 'signature',
                    type: 'ValidatorSignature'
                }
            ],
            type: 'Null'
        },
        validation_code_hash: {
            description: 'Fetch the hash of the validation code used by a para, making the given `OccupiedCoreAssumption`.',
            params: [
                {
                    name: 'paraId',
                    type: 'ParaId'
                },
                {
                    name: 'assumption',
                    type: 'OccupiedCoreAssumption'
                }
            ],
            type: 'Option<ValidationCodeHash>'
        }
    };
    const PH_V3 = {
        disputes: {
            description: 'Returns all onchain disputes.',
            params: [],
            type: 'Vec<(SessionIndex, CandidateHash, DisputeState)>'
        }
    };
    const PH_V4 = {
        session_executor_params: {
            description: 'Returns execution parameters for the session.',
            params: [
                {
                    name: 'sessionIndex',
                    type: 'SessionIndex'
                }
            ],
            type: 'Option<ExecutorParams>'
        }
    };
    const PH_V5 = {
        key_ownership_proof: {
            description: 'Returns a merkle proof of a validator session key',
            params: [
                {
                    name: 'validatorId',
                    type: 'ValidatorId'
                }
            ],
            type: 'Option<OpaqueKeyOwnershipProof>'
        },
        submit_report_dispute_lost: {
            description: 'Submit an unsigned extrinsic to slash validators who lost a dispute about a candidate of a past session',
            params: [
                {
                    name: 'disputeProof',
                    type: 'DisputeProof'
                },
                {
                    name: 'keyOwnershipProof',
                    type: 'OpaqueKeyOwnershipProof'
                }
            ],
            type: 'Option<Null>'
        },
        unapplied_slashes: {
            description: 'Returns a list of validators that lost a past session dispute and need to be slashed',
            params: [],
            type: 'Vec<(SessionIndex, CandidateHash, PendingSlashes)>'
        }
    };
    const PH_V6 = {
        minimum_backing_votes: {
            description: 'Get the minimum number of backing votes for a parachain candidate. This is a staging method! Do not use on production runtimes!',
            params: [],
            type: 'u32'
        }
    };
    const PH_V7 = {
        async_backing_params: {
            description: 'Returns candidate\'s acceptance limitations for asynchronous backing for a relay parent',
            params: [],
            type: 'AsyncBackingParams'
        },
        para_backing_state: {
            description: 'Returns the state of parachain backing for a given para',
            params: [
                {
                    name: 'paraId',
                    type: 'ParaId'
                }
            ],
            type: 'Option<BackingState>'
        }
    };
    const PH_V8 = {
        disabled_validators: {
            description: 'Returns a list of all disabled validators at the given block',
            params: [],
            type: 'ValidatorIndex'
        }
    };
    const PH_V9 = {
        node_features: {
            description: 'Get node features. This is a staging method! Do not use on production runtimes!',
            params: [],
            type: 'NodeFeatures'
        }
    };
    const PH_V10 = {
        approval_voting_params: {
            description: 'Approval voting configuration parameters',
            params: [],
            type: 'ApprovalVotingParams'
        }
    };
    const PH_V11 = {
        claim_queue: {
            description: 'Claim queue',
            params: [],
            type: 'BTreeMap<CoreIndex, Vec<u32>>'
        }
    };
    const runtime$9 = {
        ParachainHost: [
            {
                methods: {
                    ...PH_V1_TO_V2,
                    ...PH_V2_TO_V3,
                    ...PH_V3,
                    ...PH_V4,
                    ...PH_V5,
                    ...PH_V6,
                    ...PH_V7,
                    ...PH_V8,
                    ...PH_V9,
                    ...PH_V10,
                    ...PH_V11
                },
                version: 11
            },
            {
                methods: {
                    ...PH_V1_TO_V2,
                    ...PH_V2_TO_V3,
                    ...PH_V3,
                    ...PH_V4,
                    ...PH_V5,
                    ...PH_V6,
                    ...PH_V7,
                    ...PH_V8,
                    ...PH_V9,
                    ...PH_V10
                },
                version: 10
            },
            {
                methods: {
                    ...PH_V1_TO_V2,
                    ...PH_V2_TO_V3,
                    ...PH_V3,
                    ...PH_V4,
                    ...PH_V5,
                    ...PH_V6,
                    ...PH_V7,
                    ...PH_V8,
                    ...PH_V9
                },
                version: 9
            },
            {
                methods: {
                    ...PH_V1_TO_V2,
                    ...PH_V2_TO_V3,
                    ...PH_V3,
                    ...PH_V4,
                    ...PH_V5,
                    ...PH_V6,
                    ...PH_V7,
                    ...PH_V8
                },
                version: 8
            },
            {
                methods: {
                    ...PH_V1_TO_V2,
                    ...PH_V2_TO_V3,
                    ...PH_V3,
                    ...PH_V4,
                    ...PH_V5,
                    ...PH_V6,
                    ...PH_V7
                },
                version: 7
            },
            {
                methods: {
                    ...PH_V1_TO_V2,
                    ...PH_V2_TO_V3,
                    ...PH_V3,
                    ...PH_V4,
                    ...PH_V5,
                    ...PH_V6
                },
                version: 6
            },
            {
                methods: {
                    ...PH_V1_TO_V2,
                    ...PH_V2_TO_V3,
                    ...PH_V3,
                    ...PH_V4,
                    ...PH_V5
                },
                version: 5
            },
            {
                methods: {
                    ...PH_V1_TO_V2,
                    ...PH_V2_TO_V3,
                    ...PH_V3,
                    ...PH_V4
                },
                version: 4
            },
            {
                methods: {
                    ...PH_V1_TO_V2,
                    ...PH_V2_TO_V3,
                    ...PH_V3
                },
                version: 3
            },
            {
                methods: {
                    ...PH_V1_TO_V2,
                    ...PH_V2_TO_V3
                },
                version: 2
            },
            {
                methods: {
                    session_info: {
                        description: 'Get the session info for the given session, if stored.',
                        params: [
                            {
                                name: 'index',
                                type: 'SessionIndex'
                            }
                        ],
                        type: 'Option<OldV1SessionInfo>'
                    },
                    ...PH_V1_TO_V2
                },
                version: 1
            }
        ]
    };

    const SlotRange10 = {
        _enum: ['ZeroZero', 'ZeroOne', 'ZeroTwo', 'ZeroThree', 'OneOne', 'OneTwo', 'OneThree', 'TwoTwo', 'TwoThree', 'ThreeThree']
    };
    const SlotRange = {
        _enum: ['ZeroZero', 'ZeroOne', 'ZeroTwo', 'ZeroThree', 'ZeroFour', 'ZeroFive', 'ZeroSix', 'ZeroSeven', 'OneOne', 'OneTwo', 'OneThree', 'OneFour', 'OneFive', 'OneSix', 'OneSeven', 'TwoTwo', 'TwoThree', 'TwoFour', 'TwoFive', 'TwoSix', 'TwoSeven', 'ThreeThree', 'ThreeFour', 'ThreeFive', 'ThreeSix', 'ThreeSeven', 'FourFour', 'FourFive', 'FourSix', 'FourSeven', 'FiveFive', 'FiveSix', 'FiveSeven', 'SixSix', 'SixSeven', 'SevenSeven']
    };
    const oldTypes = {
        Bidder: {
            _enum: {
                New: 'NewBidder',
                Existing: 'ParaId'
            }
        },
        IncomingParachain: {
            _enum: {
                Unset: 'NewBidder',
                Fixed: 'IncomingParachainFixed',
                Deploy: 'IncomingParachainDeploy'
            }
        },
        IncomingParachainDeploy: {
            code: 'ValidationCode',
            initialHeadData: 'HeadData'
        },
        IncomingParachainFixed: {
            codeHash: 'Hash',
            codeSize: 'u32',
            initialHeadData: 'HeadData'
        },
        NewBidder: {
            who: 'AccountId',
            sub: 'SubId'
        },
        SubId: 'u32'
    };
    const slotTypes = {
        ...oldTypes,
        AuctionIndex: 'u32',
        LeasePeriod: 'BlockNumber',
        LeasePeriodOf: 'BlockNumber',
        SlotRange10,
        SlotRange,
        WinningData10: `[WinningDataEntry; ${SlotRange10._enum.length}]`,
        WinningData: `[WinningDataEntry; ${SlotRange._enum.length}]`,
        WinningDataEntry: 'Option<(AccountId, ParaId, BalanceOf)>',
        WinnersData10: 'Vec<WinnersDataTuple10>',
        WinnersData: 'Vec<WinnersDataTuple>',
        WinnersDataTuple10: '(AccountId, ParaId, BalanceOf, SlotRange10)',
        WinnersDataTuple: '(AccountId, ParaId, BalanceOf, SlotRange)'
    };

    const proposeTypes = {
        ParachainProposal: {
            proposer: 'AccountId',
            genesisHead: 'HeadData',
            validators: 'Vec<ValidatorId>',
            name: 'Bytes',
            balance: 'Balance'
        },
        RegisteredParachainInfo: {
            validators: 'Vec<ValidatorId>',
            proposer: 'AccountId'
        }
    };
    const cumulusTypes = {
        ServiceQuality: {
            _enum: ['Ordered', 'Fast']
        }
    };
    const disputeTypes = {
        DisputeLocation: {
            _enum: ['Local', 'Remote']
        },
        DisputeResult: {
            _enum: ['Valid', 'Invalid']
        },
        DisputeState: {
            validatorsFor: 'BitVec',
            validatorsAgainst: 'BitVec',
            start: 'BlockNumber',
            concludedAt: 'Option<BlockNumber>'
        },
        DisputeStatement: {
            _enum: {
                Valid: 'ValidDisputeStatementKind',
                Invalid: 'InvalidDisputeStatementKind'
            }
        },
        DisputeStatementSet: {
            candidateHash: 'CandidateHash',
            session: 'SessionIndex',
            statements: 'Vec<(DisputeStatement, ParaValidatorIndex, ValidatorSignature)>'
        },
        ExecutorParam: {
            _enum: {
                Phantom: 'Null',
                MaxMemoryPages: 'u32',
                StackLogicalMax: 'u32',
                StackNativeMax: 'u32',
                PrecheckingMaxMemory: 'u64',
                PvfPrepTimeout: '(PvfPrepTimeoutKind, u64)',
                PvfExecTimeout: '(PvfExecTimeoutKind, u64)'
            }
        },
        ExecutorParamsHash: 'Hash',
        ExecutorParams: 'Vec<ExecutorParam>',
        ExplicitDisputeStatement: {
            valid: 'bool',
            candidateHash: 'CandidateHash',
            session: 'SessionIndex'
        },
        InvalidDisputeStatementKind: {
            _enum: ['Explicit']
        },
        MultiDisputeStatementSet: 'Vec<DisputeStatementSet>',
        PvfExecTimeoutKind: {
            _enum: ['Backing', 'Approval']
        },
        PvfPrepTimeoutKind: {
            _enum: ['Precheck', 'Lenient']
        },
        ValidDisputeStatementKind: {
            _enum: {
                Explicit: 'Null',
                BackingSeconded: 'Hash',
                BackingValid: 'Hash',
                ApprovalChecking: 'Null'
            }
        }
    };
    const definitions$j = {
        rpc: {},
        runtime: runtime$9,
        types: {
            ...cumulusTypes,
            ...disputeTypes,
            ...hrmpTypes,
            ...proposeTypes,
            ...slotTypes,
            AbridgedCandidateReceipt: {
                parachainIndex: 'ParaId',
                relayParent: 'Hash',
                headData: 'HeadData',
                collator: 'CollatorId',
                signature: 'CollatorSignature',
                povBlockHash: 'Hash',
                commitments: 'CandidateCommitments'
            },
            AbridgedHostConfiguration: {
                maxCodeSize: 'u32',
                maxHeadDataSize: 'u32',
                maxUpwardQueueCount: 'u32',
                maxUpwardQueueSize: 'u32',
                maxUpwardMessageSize: 'u32',
                maxUpwardMessageNumPerCandidate: 'u32',
                hrmpMaxMessageNumPerCandidate: 'u32',
                validationUpgradeFrequency: 'BlockNumber',
                validationUpgradeDelay: 'BlockNumber'
            },
            AbridgedHrmpChannel: {
                maxCapacity: 'u32',
                maxTotalSize: 'u32',
                maxMessageSize: 'u32',
                msgCount: 'u32',
                totalSize: 'u32',
                mqcHead: 'Option<Hash>'
            },
            ApprovalVotingParams: {
                maxApprovalCoalesceCount: 'u32'
            },
            AssignmentId: 'AccountId',
            AssignmentKind: {
                _enum: {
                    Parachain: 'Null',
                    Parathread: '(CollatorId, u32)'
                }
            },
            AsyncBackingParams: {
                maxCandidateDepth: 'u32',
                allowedAncestryLen: 'u32'
            },
            AttestedCandidate: {
                candidate: 'AbridgedCandidateReceipt',
                validityVotes: 'Vec<ValidityAttestation>',
                validatorIndices: 'BitVec'
            },
            AuthorityDiscoveryId: 'AccountId',
            AvailabilityBitfield: 'BitVec',
            AvailabilityBitfieldRecord: {
                bitfield: 'AvailabilityBitfield',
                submittedTt: 'BlockNumber'
            },
            BackedCandidate: {
                candidate: 'CommittedCandidateReceipt',
                validityVotes: 'Vec<ValidityAttestation>',
                validatorIndices: 'BitVec'
            },
            BackingState: {
                constraints: 'Constraints',
                pendingAvailability: 'Vec<CandidatePendingAvailability>'
            },
            BufferedSessionChange: {
                applyAt: 'BlockNumber',
                validators: 'Vec<ValidatorId>',
                queued: 'Vec<ValidatorId>',
                sessionIndex: 'SessionIndex'
            },
            CandidateCommitments: {
                upwardMessages: 'Vec<UpwardMessage>',
                horizontalMessages: 'Vec<OutboundHrmpMessage>',
                newValidationCode: 'Option<ValidationCode>',
                headData: 'HeadData',
                processedDownwardMessages: 'u32',
                hrmpWatermark: 'BlockNumber'
            },
            CandidateDescriptor: {
                paraId: 'ParaId',
                relayParent: 'RelayChainHash',
                collatorId: 'CollatorId',
                persistedValidationDataHash: 'Hash',
                povHash: 'Hash',
                erasureRoot: 'Hash',
                signature: 'CollatorSignature',
                paraHead: 'Hash',
                validationCodeHash: 'ValidationCodeHash'
            },
            CandidateEvent: {
                _enum: {
                    CandidateBacked: '(CandidateReceipt, HeadData, CoreIndex, GroupIndex)',
                    CandidateIncluded: '(CandidateReceipt, HeadData, CoreIndex, GroupIndex)',
                    CandidateTimedOut: '(CandidateReceipt, HeadData, CoreIndex)'
                }
            },
            CandidateHash: 'Hash',
            CandidateInfo: {
                who: 'AccountId',
                deposit: 'Balance'
            },
            CandidatePendingAvailability: {
                core: 'CoreIndex',
                hash: 'CandidateHash',
                descriptor: 'CandidateDescriptor',
                availabilityVotes: 'BitVec',
                backers: 'BitVec',
                relayParentNumber: 'BlockNumber',
                backedInNumber: 'BlockNumber',
                backingGroup: 'GroupIndex'
            },
            CandidateReceipt: {
                descriptor: 'CandidateDescriptor',
                commitmentsHash: 'Hash'
            },
            GlobalValidationData: {
                maxCodeSize: 'u32',
                maxHeadDataSize: 'u32',
                blockNumber: 'BlockNumber'
            },
            CollatorId: 'H256',
            CollatorSignature: 'Signature',
            CommittedCandidateReceipt: {
                descriptor: 'CandidateDescriptor',
                commitments: 'CandidateCommitments'
            },
            Constraints: {
                minRelayParentNumber: 'BlockNumber',
                maxPovSize: 'u32',
                maxCodeSize: 'u32',
                umpRemaining: 'u32',
                umpRemainingBytes: 'u32',
                maxUmpNumPerCandidate: 'u32',
                dmpRemainingMessages: 'Vec<BlockNumber>',
                hrmpInbound: 'InboundHrmpLimitations',
                hrmpChannelsOut: 'HashMap<ParaId, OutboundHrmpChannelLimitations>',
                maxHrmpNumPerCandidate: 'u32',
                requiredParent: 'HeadData',
                validationCodeHash: 'ValidationCodeHash',
                upgradeRestriction: 'Option<UpgradeRestriction>',
                futureValidationCode: 'Option<(BlockNumber, ValidationCodeHash)>'
            },
            CoreAssignment: {
                core: 'CoreIndex',
                paraId: 'ParaId',
                kind: 'AssignmentKind',
                groupIdx: 'GroupIndex'
            },
            CoreIndex: 'u32',
            CoreOccupied: {
                _enum: {
                    Parathread: 'ParathreadEntry',
                    Parachain: 'Null'
                }
            },
            CoreState: {
                _enum: {
                    Occupied: 'OccupiedCore',
                    Scheduled: 'ScheduledCore',
                    Free: 'Null'
                }
            },
            DisputeProof: {
                timeSlot: 'DisputesTimeSlot',
                kind: 'SlashingOffenceKind',
                validatorIndex: 'ValidatorIndex',
                validatorId: 'ValidatorId'
            },
            DisputesTimeSlot: {
                sessionIndex: 'SessionIndex',
                candidateHash: 'CandidateHash'
            },
            DoubleVoteReport: {
                identity: 'ValidatorId',
                first: '(Statement, ValidatorSignature)',
                second: '(Statement, ValidatorSignature)',
                proof: 'MembershipProof',
                signingContext: 'SigningContext'
            },
            DownwardMessage: 'Bytes',
            GroupIndex: 'u32',
            GroupRotationInfo: {
                sessionStartBlock: 'BlockNumber',
                groupRotationFrequency: 'BlockNumber',
                now: 'BlockNumber'
            },
            GlobalValidationSchedule: {
                maxCodeSize: 'u32',
                maxHeadDataSize: 'u32',
                blockNumber: 'BlockNumber'
            },
            HeadData: 'Bytes',
            HostConfiguration: {
                maxCodeSize: 'u32',
                maxHeadDataSize: 'u32',
                maxUpwardQueueCount: 'u32',
                maxUpwardQueueSize: 'u32',
                maxUpwardMessageSize: 'u32',
                maxUpwardMessageNumPerCandidate: 'u32',
                hrmpMaxMessageNumPerCandidate: 'u32',
                validationUpgradeFrequency: 'BlockNumber',
                validationUpgradeDelay: 'BlockNumber',
                maxPovSize: 'u32',
                maxDownwardMessageSize: 'u32',
                preferredDispatchableUpwardMessagesStepWeight: 'Weight',
                hrmpMaxParachainOutboundChannels: 'u32',
                hrmpMaxParathreadOutboundChannels: 'u32',
                hrmpOpenRequestTtl: 'u32',
                hrmpSenderDeposit: 'Balance',
                hrmpRecipientDeposit: 'Balance',
                hrmpChannelMaxCapacity: 'u32',
                hrmpChannelMaxTotalSize: 'u32',
                hrmpMaxParachainInboundChannels: 'u32',
                hrmpMaxParathreadInboundChannels: 'u32',
                hrmpChannelMaxMessageSize: 'u32',
                codeRetentionPeriod: 'BlockNumber',
                parathreadCores: 'u32',
                parathreadRetries: 'u32',
                groupRotationFrequency: 'BlockNumber',
                chainAvailabilityPeriod: 'BlockNumber',
                threadAvailabilityPeriod: 'BlockNumber',
                schedulingLookahead: 'u32',
                maxValidatorsPerCore: 'Option<u32>',
                maxValidators: 'Option<u32>',
                disputePeriod: 'SessionIndex',
                disputePostConclusionAcceptancePeriod: 'BlockNumber',
                disputeMaxSpamSlots: 'u32',
                disputeConclusionByTimeOutPeriod: 'BlockNumber',
                noShowSlots: 'u32',
                nDelayTranches: 'u32',
                zerothDelayTrancheWidth: 'u32',
                neededApprovals: 'u32',
                relayVrfModuloSamples: 'u32'
            },
            InboundDownwardMessage: {
                pubSentAt: 'BlockNumber',
                pubMsg: 'DownwardMessage'
            },
            InboundHrmpMessage: {
                sentAt: 'BlockNumber',
                data: 'Bytes'
            },
            InboundHrmpLimitations: {
                validWatermarks: 'Vec<BlockNumber>'
            },
            InboundHrmpMessages: 'Vec<InboundHrmpMessage>',
            LocalValidationData: {
                parentHead: 'HeadData',
                balance: 'Balance',
                codeUpgradeAllowed: 'Option<BlockNumber>'
            },
            MessageIngestionType: {
                downwardMessages: 'Vec<InboundDownwardMessage>',
                horizontalMessages: 'BTreeMap<ParaId, InboundHrmpMessages>'
            },
            MessageQueueChain: 'RelayChainHash',
            NodeFeatures: 'BitVec',
            OccupiedCore: {
                nextUpOnAvailable: 'Option<ScheduledCore>',
                occupiedSince: 'BlockNumber',
                timeOutAt: 'BlockNumber',
                nextUpOnTimeOut: 'Option<ScheduledCore>',
                availability: 'BitVec',
                groupResponsible: 'GroupIndex',
                candidateHash: 'CandidateHash',
                candidateDescriptor: 'CandidateDescriptor'
            },
            OccupiedCoreAssumption: {
                _enum: ['Included,', 'TimedOut', 'Free']
            },
            OutboundHrmpChannelLimitations: {
                bytesRemaining: 'u32',
                messagesRemaining: 'u32'
            },
            OutboundHrmpMessage: {
                recipient: 'u32',
                data: 'Bytes'
            },
            PendingSlashes: {
                _alias: {
                    slashKeys: 'keys'
                },
                slashKeys: 'BTreeMap<ValidatorIndex, ValidatorId>',
                kind: 'SlashingOffenceKind'
            },
            ParachainDispatchOrigin: {
                _enum: ['Signed', 'Parachain', 'Root']
            },
            ParachainInherentData: {
                validationData: 'PersistedValidationData',
                relayChainState: 'StorageProof',
                downwardMessages: 'Vec<InboundDownwardMessage>',
                horizontalMessages: 'BTreeMap<ParaId, VecInboundHrmpMessage>'
            },
            ParachainsInherentData: {
                bitfields: 'SignedAvailabilityBitfields',
                backedCandidates: 'Vec<BackedCandidate>',
                disputes: 'MultiDisputeStatementSet',
                parentHeader: 'Header'
            },
            ParaGenesisArgs: {
                genesisHead: 'Bytes',
                validationCode: 'Bytes',
                parachain: 'bool'
            },
            ParaId: 'u32',
            ParaInfo: {
                manager: 'AccountId',
                deposit: 'Balance',
                locked: 'bool'
            },
            ParaLifecycle: {
                _enum: ['Onboarding', 'Parathread', 'Parachain', 'UpgradingToParachain', 'DowngradingToParathread', 'OutgoingParathread', 'OutgoingParachain']
            },
            ParaPastCodeMeta: {
                upgradeTimes: 'Vec<ReplacementTimes>',
                lastPruned: 'Option<BlockNumber>'
            },
            ParaScheduling: {
                _enum: ['Always', 'Dynamic']
            },
            ParathreadClaim: '(ParaId, CollatorId)',
            ParathreadClaimQueue: {
                queue: 'Vec<QueuedParathread>',
                nextCoreOffset: 'u32'
            },
            ParathreadEntry: {
                claim: 'ParathreadClaim',
                retries: 'u32'
            },
            ParaValidatorIndex: 'u32',
            PersistedValidationData: {
                parentHead: 'HeadData',
                relayParentNumber: 'RelayChainBlockNumber',
                relayParentStorageRoot: 'Hash',
                maxPovSize: 'u32'
            },
            PvfCheckStatement: {
                accept: 'bool',
                subject: 'ValidationCodeHash',
                sessionIndex: 'SessionIndex',
                validatorIndex: 'ParaValidatorIndex'
            },
            QueuedParathread: {
                claim: 'ParathreadEntry',
                coreOffset: 'u32'
            },
            RelayBlockNumber: 'u32',
            RelayChainBlockNumber: 'RelayBlockNumber',
            RelayHash: 'Hash',
            RelayChainHash: 'RelayHash',
            Remark: '[u8; 32]',
            ReplacementTimes: {
                expectedAt: 'BlockNumber',
                activatedAt: 'BlockNumber'
            },
            Retriable: {
                _enum: {
                    Never: 'Null',
                    WithRetries: 'u32'
                }
            },
            ScheduledCore: {
                paraId: 'ParaId',
                collator: 'Option<CollatorId>'
            },
            Scheduling: {
                _enum: ['Always', 'Dynamic']
            },
            ScrapedOnChainVotes: {
                session: 'SessionIndex',
                backingValidatorsPerCandidate: 'Vec<(CandidateReceipt, Vec<(ParaValidatorIndex, ValidityAttestation)>)>',
                disputes: 'MultiDisputeStatementSet'
            },
            SessionInfo: {
                activeValidatorIndices: 'Vec<ParaValidatorIndex>',
                randomSeed: '[u8; 32]',
                disputePeriod: 'SessionIndex',
                validators: 'Vec<ValidatorId>',
                discoveryKeys: 'Vec<AuthorityDiscoveryId>',
                assignmentKeys: 'Vec<AssignmentId>',
                validatorGroups: 'Vec<Vec<ValidatorIndex>>',
                nCores: 'u32',
                zerothDelayTrancheWidth: 'u32',
                relayVrfModuloSamples: 'u32',
                nDelayTranches: 'u32',
                noShowSlots: 'u32',
                neededApprovals: 'u32'
            },
            OldV1SessionInfo: {
                validators: 'Vec<ValidatorId>',
                discoveryKeys: 'Vec<AuthorityDiscoveryId>',
                assignmentKeys: 'Vec<AssignmentId>',
                validatorGroups: 'Vec<Vec<ParaValidatorIndex>>',
                nCores: 'u32',
                zerothDelayTrancheWidth: 'u32',
                relayVrfModuloSamples: 'u32',
                nDelayTranches: 'u32',
                noShowSlots: 'u32',
                neededApprovals: 'u32'
            },
            SessionInfoValidatorGroup: 'Vec<ParaValidatorIndex>',
            SignedAvailabilityBitfield: {
                payload: 'BitVec',
                validatorIndex: 'ParaValidatorIndex',
                signature: 'ValidatorSignature'
            },
            SignedAvailabilityBitfields: 'Vec<SignedAvailabilityBitfield>',
            SigningContext: {
                sessionIndex: 'SessionIndex',
                parentHash: 'Hash'
            },
            SlashingOffenceKind: {
                _enum: ['ForInvalid', 'AgainstValid']
            },
            Statement: {
                _enum: {
                    Never: 'Null',
                    Candidate: 'Hash',
                    Valid: 'Hash',
                    Invalid: 'Hash'
                }
            },
            TransientValidationData: {
                maxCodeSize: 'u32',
                maxHeadDataSize: 'u32',
                balance: 'Balance',
                codeUpgradeAllowed: 'Option<BlockNumber>',
                dmqLength: 'u32'
            },
            UpgradeGoAhead: {
                _enum: ['Abort', 'GoAhead']
            },
            UpgradeRestriction: {
                _enum: ['Present']
            },
            UpwardMessage: 'Bytes',
            ValidationFunctionParams: {
                maxCodeSize: 'u32',
                relayChainHeight: 'RelayChainBlockNumber',
                codeUpgradeAllowed: 'Option<RelayChainBlockNumber>'
            },
            ValidationCode: 'Bytes',
            ValidationCodeHash: 'Hash',
            ValidationData: {
                persisted: 'PersistedValidationData',
                transient: 'TransientValidationData'
            },
            ValidationDataType: {
                validationData: 'ValidationData',
                relayChainState: 'Vec<Bytes>'
            },
            ValidatorSignature: 'Signature',
            ValidityAttestation: {
                _enum: {
                    Never: 'Null',
                    Implicit: 'ValidatorSignature',
                    Explicit: 'ValidatorSignature'
                }
            },
            MessagingStateSnapshot: {
                relayDispatchQueueSize: '(u32, u32)',
                egressChannels: 'Vec<MessagingStateSnapshotEgressEntry>'
            },
            MessagingStateSnapshotEgressEntry: '(ParaId, AbridgedHrmpChannel)',
            SystemInherentData: 'ParachainInherentData',
            VecInboundHrmpMessage: 'Vec<InboundHrmpMessage>'
        }
    };

    const definitions$i = {
        rpc: {},
        types: {
            Approvals: '[bool; 4]'
        }
    };

    const definitions$h = {
        rpc: {},
        types: {
            AccountStatus: {
                validity: 'AccountValidity',
                freeBalance: 'Balance',
                lockedBalance: 'Balance',
                signature: 'Vec<u8>',
                vat: 'Permill'
            },
            AccountValidity: {
                _enum: ['Invalid', 'Initiated', 'Pending', 'ValidLow', 'ValidHigh', 'Completed']
            }
        }
    };

    const runtime$8 = {
        DryRunApi: [
            {
                methods: {
                    dry_run_call: {
                        description: 'Dry run call',
                        params: [
                            {
                                name: 'origin',
                                type: 'OriginCaller'
                            },
                            {
                                name: 'call',
                                type: 'RuntimeCall'
                            }
                        ],
                        type: 'Result<CallDryRunEffects, XcmDryRunApiError>'
                    },
                    dry_run_xcm: {
                        description: 'Dry run XCM program',
                        params: [
                            {
                                name: 'originLocation',
                                type: 'VersionedMultiLocation'
                            },
                            {
                                name: 'xcm',
                                type: 'VersionedXcm'
                            }
                        ],
                        type: 'Result<XcmDryRunEffects, XcmDryRunApiError>'
                    }
                },
                version: 1
            },
            {
                methods: {
                    dry_run_call: {
                        description: 'Dry run call',
                        params: [
                            {
                                name: 'origin',
                                type: 'OriginCaller'
                            },
                            {
                                name: 'call',
                                type: 'RuntimeCall'
                            },
                            {
                                name: 'resultXcmsVersion',
                                type: 'u32'
                            }
                        ],
                        type: 'Result<CallDryRunEffects, XcmDryRunApiError>'
                    },
                    dry_run_xcm: {
                        description: 'Dry run XCM program',
                        params: [
                            {
                                name: 'originLocation',
                                type: 'VersionedMultiLocation'
                            },
                            {
                                name: 'xcm',
                                type: 'VersionedXcm'
                            }
                        ],
                        type: 'Result<XcmDryRunEffects, XcmDryRunApiError>'
                    }
                },
                version: 2
            }
        ]
    };

    const definitions$g = {
        rpc: {},
        runtime: runtime$8,
        types: {
            PostDispatchInfo: {
                actualWeight: 'Option<Weight>',
                paysFee: 'Pays'
            },
            DispatchResultWithPostInfo: 'Result<PostDispatchInfo, DispatchError>',
            CallDryRunEffects: {
                executionResult: 'DispatchResultWithPostInfo',
                emittedEvents: 'Vec<Event>',
                localXcm: 'Option<VersionedXcm>',
                forwardedXcms: 'Vec<(VersionedMultiLocation, Vec<VersionedXcm>)>'
            },
            XcmDryRunEffects: {
                executionResult: 'OutcomeV4',
                emittedEvents: 'Vec<Event>',
                forwardedXcms: 'Vec<(VersionedMultiLocation, Vec<VersionedXcm>)>'
            },
            XcmDryRunApiError: {
                _enum: [
                    'Unimplemented',
                    'VersionedConversionFailed'
                ]
            }
        }
    };

    exports.TypeDefInfo = void 0;
    (function (TypeDefInfo) {
        TypeDefInfo[TypeDefInfo["BTreeMap"] = 0] = "BTreeMap";
        TypeDefInfo[TypeDefInfo["BTreeSet"] = 1] = "BTreeSet";
        TypeDefInfo[TypeDefInfo["Compact"] = 2] = "Compact";
        TypeDefInfo[TypeDefInfo["DoNotConstruct"] = 3] = "DoNotConstruct";
        TypeDefInfo[TypeDefInfo["Enum"] = 4] = "Enum";
        TypeDefInfo[TypeDefInfo["HashMap"] = 5] = "HashMap";
        TypeDefInfo[TypeDefInfo["Int"] = 6] = "Int";
        TypeDefInfo[TypeDefInfo["Linkage"] = 7] = "Linkage";
        TypeDefInfo[TypeDefInfo["Null"] = 8] = "Null";
        TypeDefInfo[TypeDefInfo["Option"] = 9] = "Option";
        TypeDefInfo[TypeDefInfo["Plain"] = 10] = "Plain";
        TypeDefInfo[TypeDefInfo["Range"] = 11] = "Range";
        TypeDefInfo[TypeDefInfo["RangeInclusive"] = 12] = "RangeInclusive";
        TypeDefInfo[TypeDefInfo["Result"] = 13] = "Result";
        TypeDefInfo[TypeDefInfo["Set"] = 14] = "Set";
        TypeDefInfo[TypeDefInfo["Si"] = 15] = "Si";
        TypeDefInfo[TypeDefInfo["Struct"] = 16] = "Struct";
        TypeDefInfo[TypeDefInfo["Tuple"] = 17] = "Tuple";
        TypeDefInfo[TypeDefInfo["UInt"] = 18] = "UInt";
        TypeDefInfo[TypeDefInfo["Vec"] = 19] = "Vec";
        TypeDefInfo[TypeDefInfo["VecFixed"] = 20] = "VecFixed";
        TypeDefInfo[TypeDefInfo["WrapperKeepOpaque"] = 21] = "WrapperKeepOpaque";
        TypeDefInfo[TypeDefInfo["WrapperOpaque"] = 22] = "WrapperOpaque";
    })(exports.TypeDefInfo || (exports.TypeDefInfo = {}));

    function hasEq(o) {
        return util.isFunction(o.eq);
    }

    function compareArray(a, b) {
        if (Array.isArray(b)) {
            return (a.length === b.length) && util.isUndefined(a.find((v, index) => hasEq(v)
                ? !v.eq(b[index])
                : v !== b[index]));
        }
        return false;
    }

    class AbstractArray extends Array {
        registry;
        createdAtHash;
        initialU8aLength;
        isStorageFallback;
        static get [Symbol.species]() {
            return Array;
        }
        constructor(registry, length) {
            super(length);
            this.registry = registry;
        }
        get encodedLength() {
            const count = this.length;
            let total = util.compactToU8a(count).length;
            for (let i = 0; i < count; i++) {
                total += this[i].encodedLength;
            }
            return total;
        }
        get hash() {
            return this.registry.hash(this.toU8a());
        }
        get isEmpty() {
            return this.length === 0;
        }
        get length() {
            return super.length;
        }
        eq(other) {
            return compareArray(this, other);
        }
        inspect() {
            return {
                inner: this.inspectInner(),
                outer: [util.compactToU8a(this.length)]
            };
        }
        inspectInner() {
            const count = this.length;
            const inner = new Array(count);
            for (let i = 0; i < count; i++) {
                inner[i] = this[i].inspect();
            }
            return inner;
        }
        toArray() {
            return Array.from(this);
        }
        toHex() {
            return util.u8aToHex(this.toU8a());
        }
        toHuman(isExtended, disableAscii) {
            const count = this.length;
            const result = new Array(count);
            for (let i = 0; i < count; i++) {
                result[i] = this[i] && this[i].toHuman(isExtended, disableAscii);
            }
            return result;
        }
        toJSON() {
            const count = this.length;
            const result = new Array(count);
            for (let i = 0; i < count; i++) {
                result[i] = this[i] && this[i].toJSON();
            }
            return result;
        }
        toPrimitive(disableAscii) {
            const count = this.length;
            const result = new Array(count);
            for (let i = 0; i < count; i++) {
                result[i] = this[i] && this[i].toPrimitive(disableAscii);
            }
            return result;
        }
        toString() {
            const count = this.length;
            const result = new Array(count);
            for (let i = 0; i < count; i++) {
                result[i] = this[i].toString();
            }
            return `[${result.join(', ')}]`;
        }
        toU8a(isBare) {
            const encoded = this.toU8aInner();
            return isBare
                ? util.u8aConcatStrict(encoded)
                : util.u8aConcatStrict([util.compactToU8a(this.length), ...encoded]);
        }
        toU8aInner(isBare) {
            const count = this.length;
            const encoded = new Array(count);
            for (let i = 0; i < count; i++) {
                encoded[i] = this[i].toU8a(isBare);
            }
            return encoded;
        }
    }

    class AbstractBase {
        registry;
        createdAtHash;
        initialU8aLength;
        isStorageFallback;
        #raw;
        constructor(registry, value, initialU8aLength) {
            this.initialU8aLength = initialU8aLength;
            this.#raw = value;
            this.registry = registry;
        }
        get encodedLength() {
            return this.toU8a().length;
        }
        get hash() {
            return this.registry.hash(this.toU8a());
        }
        get inner() {
            return this.#raw;
        }
        get isEmpty() {
            return this.#raw.isEmpty;
        }
        eq(other) {
            return this.#raw.eq(other);
        }
        inspect() {
            return this.#raw.inspect();
        }
        toHex(isLe) {
            return this.#raw.toHex(isLe);
        }
        toHuman(isExtended, disableAscii) {
            return this.#raw.toHuman(isExtended, disableAscii);
        }
        toJSON() {
            return this.#raw.toJSON();
        }
        toPrimitive(disableAscii) {
            return this.#raw.toPrimitive(disableAscii);
        }
        toString() {
            return this.#raw.toString();
        }
        toU8a(isBare) {
            return this.#raw.toU8a(isBare);
        }
        unwrap() {
            return this.#raw;
        }
        valueOf() {
            return this.#raw;
        }
    }

    const DEFAULT_UINT_BITS = 64;
    const MAX_NUMBER_BITS = 52;
    const MUL_P = new util.BN(1_00_00);
    const FORMATTERS = [
        ['Perquintill', util.BN_QUINTILL],
        ['Perbill', util.BN_BILLION],
        ['Permill', util.BN_MILLION],
        ['Percent', util.BN_HUNDRED]
    ];
    function isToBn(value) {
        return util.isFunction(value.toBn);
    }
    function toPercentage(value, divisor) {
        return `${(value.mul(MUL_P).div(divisor).toNumber() / 100).toFixed(2)}%`;
    }
    function decodeAbstractInt(value, isNegative) {
        if (util.isNumber(value)) {
            if (!Number.isInteger(value) || value > Number.MAX_SAFE_INTEGER || value < Number.MIN_SAFE_INTEGER) {
                throw new Error('Number needs to be an integer <= Number.MAX_SAFE_INTEGER, i.e. 2 ^ 53 - 1');
            }
            return value;
        }
        else if (util.isString(value)) {
            if (util.isHex(value, -1, true)) {
                return util.hexToBn(value, { isLe: false, isNegative }).toString();
            }
            if (value.includes('.') || value.includes(',') || value.includes('e')) {
                throw new Error('String should not contain decimal points or scientific notation');
            }
            return value;
        }
        else if (util.isBn(value) || util.isBigInt(value)) {
            return value.toString();
        }
        else if (util.isObject(value)) {
            if (isToBn(value)) {
                return value.toBn().toString();
            }
            const keys = Object.keys(value);
            if (keys.length !== 1) {
                throw new Error('Unable to construct number from multi-key object');
            }
            return decodeAbstractInt(value[keys[0]], isNegative);
        }
        else if (!value) {
            return 0;
        }
        throw new Error(`Unable to create BN from unknown type ${typeof value}`);
    }
    class AbstractInt extends util.BN {
        registry;
        encodedLength;
        isUnsigned;
        createdAtHash;
        initialU8aLength;
        isStorageFallback;
        #bitLength;
        constructor(registry, value = 0, bitLength = DEFAULT_UINT_BITS, isSigned = false) {
            super(
            util.isU8a(value)
                ? bitLength <= 48
                    ? util.u8aToNumber(value.subarray(0, bitLength / 8), { isNegative: isSigned })
                    : util.u8aToBn(value.subarray(0, bitLength / 8), { isLe: true, isNegative: isSigned }).toString()
                : decodeAbstractInt(value, isSigned));
            this.registry = registry;
            this.#bitLength = bitLength;
            this.encodedLength = this.#bitLength / 8;
            this.initialU8aLength = this.#bitLength / 8;
            this.isUnsigned = !isSigned;
            const isNegative = this.isNeg();
            const maxBits = bitLength - (isSigned && !isNegative ? 1 : 0);
            if (isNegative && !isSigned) {
                throw new Error(`${this.toRawType()}: Negative number passed to unsigned type`);
            }
            else if (super.bitLength() > maxBits) {
                throw new Error(`${this.toRawType()}: Input too large. Found input with ${super.bitLength()} bits, expected ${maxBits}`);
            }
        }
        get hash() {
            return this.registry.hash(this.toU8a());
        }
        get isEmpty() {
            return this.isZero();
        }
        bitLength() {
            return this.#bitLength;
        }
        eq(other) {
            return super.eq(util.isHex(other)
                ? util.hexToBn(other.toString(), { isLe: false, isNegative: !this.isUnsigned })
                : util.bnToBn(other));
        }
        inspect() {
            return {
                outer: [this.toU8a()]
            };
        }
        isMax() {
            const u8a = this.toU8a().filter((b) => b === 0xff);
            return u8a.length === (this.#bitLength / 8);
        }
        toBigInt() {
            return BigInt(this.toString());
        }
        toBn() {
            return this;
        }
        toHex(isLe = false) {
            return util.bnToHex(this, {
                bitLength: this.bitLength(),
                isLe,
                isNegative: !this.isUnsigned
            });
        }
        toHuman(_isExpanded) {
            const rawType = this.toRawType();
            if (rawType === 'Balance') {
                return this.isMax()
                    ? 'everything'
                    : util.formatBalance(this, { decimals: this.registry.chainDecimals[0], withSi: true, withUnit: this.registry.chainTokens[0] });
            }
            const [, divisor] = FORMATTERS.find(([type]) => type === rawType) || [];
            return divisor
                ? toPercentage(this, divisor)
                : util.formatNumber(this);
        }
        toJSON(onlyHex = false) {
            return onlyHex || (this.#bitLength > 128) || (super.bitLength() > MAX_NUMBER_BITS)
                ? this.toHex()
                : this.toNumber();
        }
        toPrimitive() {
            return super.bitLength() > MAX_NUMBER_BITS
                ? this.toString()
                : this.toNumber();
        }
        toRawType() {
            return this instanceof this.registry.createClassUnsafe('Balance')
                ? 'Balance'
                : `${this.isUnsigned ? 'u' : 'i'}${this.bitLength()}`;
        }
        toString(base) {
            return super.toString(base);
        }
        toU8a(_isBare) {
            return util.bnToU8a(this, {
                bitLength: this.bitLength(),
                isLe: true,
                isNegative: !this.isUnsigned
            });
        }
    }

    function hasMismatch(a, b) {
        return util.isUndefined(a) || (hasEq(a)
            ? !a.eq(b)
            : a !== b);
    }
    function notEntry(value) {
        return !Array.isArray(value) || value.length !== 2;
    }
    function compareMapArray(a, b) {
        return (a.size === b.length) && !b.some((e) => notEntry(e) ||
            hasMismatch(a.get(e[0]), e[1]));
    }
    function compareMap(a, b) {
        if (Array.isArray(b)) {
            return compareMapArray(a, b);
        }
        else if (b instanceof Map) {
            return compareMapArray(a, [...b.entries()]);
        }
        else if (util.isObject(b)) {
            return compareMapArray(a, Object.entries(b));
        }
        return false;
    }

    function compareSetArray(a, b) {
        return (a.size === b.length) && !b.some((e) => !a.has(e));
    }
    function compareSet(a, b) {
        if (Array.isArray(b)) {
            return compareSetArray(a, b);
        }
        else if (b instanceof Set) {
            return compareSetArray(a, [...b.values()]);
        }
        else if (util.isObject(b)) {
            return compareSetArray(a, Object.values(b));
        }
        return false;
    }

    const MAX_DEPTH = 1024;
    function isComplexType(Type) {
        const typeName = Type.name?.toLowerCase() || '';
        return ['enum', 'hashmap', 'linkage', 'null', 'option', 'range', 'rangeinclusive', 'result', 'struct', 'tuple', 'vec', 'vecfixed'].includes(typeName);
    }
    function formatFailure(registry, fn, _result, { message }, u8a, i, count, Type, key) {
        let type = '';
        try {
            type = `: ${new Type(registry).toRawType()}`;
        }
        catch {
        }
        return `${fn}: failed at ${util.u8aToHex(u8a.subarray(0, 16))}…${key ? ` on ${key}` : ''} (index ${i + 1}/${count})${type}:: ${message}`;
    }
    function decodeU8a$7(registry, result, u8a, [Types, keys]) {
        const count = result.length;
        let offset = 0;
        let i = 0;
        try {
            while (i < count) {
                const value = new Types[i](registry, u8a.subarray(offset));
                offset += value.initialU8aLength || value.encodedLength;
                result[i] = value;
                i++;
            }
        }
        catch (error) {
            throw new Error(formatFailure(registry, 'decodeU8a', result, error, u8a.subarray(offset), i, count, Types[i], keys[i]));
        }
        return [result, offset];
    }
    function decodeU8aStruct(registry, result, u8a, [Types, keys]) {
        const count = result.length;
        let offset = 0;
        let i = 0;
        if (count > MAX_DEPTH && isComplexType(Types[i])) {
            throw new Error(`decodeU8aStruct: Maximum depth exceeded, received ${count} elements, limit ${MAX_DEPTH}`);
        }
        try {
            while (i < count) {
                const value = new Types[i](registry, u8a.subarray(offset));
                offset += value.initialU8aLength || value.encodedLength;
                result[i] = [keys[i], value];
                i++;
            }
        }
        catch (error) {
            throw new Error(formatFailure(registry, 'decodeU8aStruct', result, error, u8a.subarray(offset), i, count, Types[i], keys[i]));
        }
        return [result, offset];
    }
    function decodeU8aVec(registry, result, u8a, startAt, Type) {
        const count = result.length;
        if (count > MAX_DEPTH && isComplexType(Type)) {
            throw new Error(`decodeU8aVec: Maximum depth exceeded, received ${count} elements, limit ${MAX_DEPTH}`);
        }
        let offset = startAt;
        let i = 0;
        try {
            while (i < count) {
                const value = new Type(registry, u8a.subarray(offset));
                offset += value.initialU8aLength || value.encodedLength;
                result[i] = value;
                i++;
            }
        }
        catch (error) {
            throw new Error(formatFailure(registry, 'decodeU8aVec', result, error, u8a.subarray(offset), i, count, Type));
        }
        return [offset, offset - startAt];
    }

    const BOUNDED = ['BTreeMap', 'BTreeSet', 'HashMap', 'Vec'];
    const ALLOWED_BOXES = BOUNDED.concat(['Compact', 'DoNotConstruct', 'Int', 'Linkage', 'Range', 'RangeInclusive', 'Result', 'Opaque', 'Option', 'UInt', 'WrapperKeepOpaque', 'WrapperOpaque']);
    const BOX_PRECEDING = ['<', '(', '[', '"', ',', ' '];
    const mappings = [
        alias('<T::InherentOfflineReport as InherentOfflineReport>::Inherent', 'InherentOfflineReport', false),
        alias('VecDeque<', 'Vec<', false),
        cleanupCompact(),
        removeExtensions('Bounded', true),
        removeExtensions('Weak', false),
        removeTraits(),
        removePairOf(),
        removeWrap('Box<'),
        removeGenerics(),
        alias('String', 'Text'),
        alias('Vec<u8>', 'Bytes'),
        alias('&\\[u8\\]', 'Bytes'),
        alias("&'static\\[u8\\]", 'Bytes'),
        alias('RawAddress', 'Address'),
        alias('Lookup::Source', 'LookupSource'),
        alias('Lookup::Target', 'LookupTarget'),
        alias('exec::StorageKey', 'ContractStorageKey'),
        flattenSingleTuple(),
        removeColons(),
        trim()
    ];
    function trim() {
        return (value) => value.trim();
    }
    function findClosing(value, start) {
        let depth = 0;
        for (let i = start, count = value.length; i < count; i++) {
            if (value[i] === '>') {
                if (!depth) {
                    return i;
                }
                depth--;
            }
            else if (value[i] === '<') {
                depth++;
            }
        }
        throw new Error(`Unable to find closing matching <> on '${value}' (start ${start})`);
    }
    function alias(src, dest, withChecks = true) {
        const from = new RegExp(`(^${src}|${BOX_PRECEDING.map((box) => `\\${box}${src}`).join('|')})`, 'g');
        const to = (src) => {
            from.lastIndex = 0;
            return withChecks && BOX_PRECEDING.includes(src[0])
                ? `${src[0]}${dest}`
                : dest;
        };
        return (value) => value.replace(from, to);
    }
    function cleanupCompact() {
        return (value) => {
            if (value.includes(' as HasCompact')) {
                for (let i = 0, count = value.length; i < count; i++) {
                    if (value[i] === '<') {
                        const end = findClosing(value, i + 1) - 14;
                        if (value.substring(end, end + 14) === ' as HasCompact') {
                            value = `Compact<${value.substring(i + 1, end)}>`;
                        }
                    }
                }
            }
            return value;
        };
    }
    function flattenSingleTuple() {
        const from1 = /,\)/g;
        const from2 = /\(([^,]+)\)/;
        return (value) => {
            from1.lastIndex = 0;
            return value
                .replace(from1, ')')
                .replace(from2, '$1');
        };
    }
    function replaceTagWith(value, matcher, replacer) {
        let index = -1;
        while (true) {
            index = value.indexOf(matcher, index + 1);
            if (index === -1) {
                return value;
            }
            const start = index + matcher.length;
            const end = findClosing(value, start);
            value = `${value.substring(0, index)}${replacer(value.substring(start, end))}${value.substring(end + 1)}`;
        }
    }
    function removeExtensions(type, isSized) {
        return (value) => {
            for (let i = 0, count = BOUNDED.length; i < count; i++) {
                const tag = BOUNDED[i];
                value = replaceTagWith(value, `${type}${tag}<`, (v) => {
                    const parts = v
                        .split(',')
                        .map((s) => s.trim())
                        .filter((s) => s);
                    if (isSized) {
                        parts.pop();
                    }
                    return `${tag}<${parts.join(',')}>`;
                });
            }
            return value;
        };
    }
    function removeColons() {
        return (value) => {
            let index = 0;
            while (index !== -1) {
                index = value.indexOf('::');
                if (index === 0) {
                    value = value.substring(2);
                }
                else if (index !== -1) {
                    let start = index;
                    while (start !== -1 && !BOX_PRECEDING.includes(value[start])) {
                        start--;
                    }
                    value = `${value.substring(0, start + 1)}${value.substring(index + 2)}`;
                }
            }
            return value;
        };
    }
    function removeGenerics() {
        return (value) => {
            for (let i = 0, count = value.length; i < count; i++) {
                if (value[i] === '<') {
                    const box = ALLOWED_BOXES.find((box) => {
                        const start = i - box.length;
                        return ((start >= 0 &&
                            value.substring(start, i) === box) && (
                        start === 0 ||
                            BOX_PRECEDING.includes(value[start - 1])));
                    });
                    if (!box) {
                        const end = findClosing(value, i + 1);
                        value = `${value.substring(0, i)}${value.substring(end + 1)}`;
                    }
                }
            }
            return value;
        };
    }
    function removePairOf() {
        const replacer = (v) => `(${v},${v})`;
        return (value) => replaceTagWith(value, 'PairOf<', replacer);
    }
    function removeTraits() {
        const from1 = /\s/g;
        const from2 = /(T|Self)::/g;
        const from3 = /<(T|Self)asTrait>::/g;
        const from4 = /<Tas[a-z]+::Trait>::/g;
        const from5 = /<LookupasStaticLookup>/g;
        const from6 = /::Type/g;
        return (value) => {
            from1.lastIndex = 0;
            from2.lastIndex = 0;
            from3.lastIndex = 0;
            from4.lastIndex = 0;
            from5.lastIndex = 0;
            from6.lastIndex = 0;
            return value
                .replace(from1, '')
                .replace(from2, '')
                .replace(from3, '')
                .replace(from4, '')
                .replace(from5, 'Lookup')
                .replace(from6, '');
        };
    }
    function removeWrap(check) {
        const replacer = (v) => v;
        return (value) => replaceTagWith(value, check, replacer);
    }
    const sanitizeMap = new Map();
    function sanitize(value) {
        const startValue = value.toString();
        const memoized = sanitizeMap.get(startValue);
        if (memoized) {
            return memoized;
        }
        let result = startValue;
        for (let i = 0, count = mappings.length; i < count; i++) {
            result = mappings[i](result);
        }
        sanitizeMap.set(startValue, result);
        return result;
    }

    function isArrayLike(arg) {
        return arg instanceof Uint8Array || Array.isArray(arg);
    }
    function isEnum(arg) {
        return util.isCodec(arg) && util.isNumber(arg.index) && util.isCodec(arg.value);
    }
    function isOption(arg) {
        return util.isCodec(arg) && util.isBoolean(arg.isSome) && util.isCodec(arg.value);
    }
    function isNumberLike(arg) {
        return util.isNumber(arg) || util.isBn(arg) || util.isBigInt(arg);
    }
    function sortArray(a, b) {
        let sortRes = 0;
        const minLen = Math.min(a.length, b.length);
        for (let i = 0; i < minLen; ++i) {
            sortRes = sortAsc(a[i], b[i]);
            if (sortRes !== 0) {
                return sortRes;
            }
        }
        return a.length - b.length;
    }
    function checkForDuplicates(container, seen, arg) {
        if (util.isCodec(arg)) {
            const hex = arg.toHex();
            if (seen.has(hex)) {
                throw new Error(`Duplicate value in ${container}: ${util.stringify(arg)}`);
            }
            seen.add(hex);
        }
        return true;
    }
    function sortAsc(a, b) {
        if (isNumberLike(a) && isNumberLike(b)) {
            return util.bnToBn(a).cmp(util.bnToBn(b));
        }
        else if (a instanceof Map && b instanceof Map) {
            return sortAsc(Array.from(a.values()), Array.from(b.values()));
        }
        else if (isEnum(a) && isEnum(b)) {
            return sortAsc(a.index, b.index) || sortAsc(a.value, b.value);
        }
        else if (isOption(a) && isOption(b)) {
            return sortAsc(a.isNone ? 0 : 1, b.isNone ? 0 : 1) || sortAsc(a.value, b.value);
        }
        else if (isArrayLike(a) && isArrayLike(b)) {
            return sortArray(a, b);
        }
        else if (util.isCodec(a) && util.isCodec(b)) {
            return sortAsc(a.toU8a(true), b.toU8a(true));
        }
        throw new Error(`Attempting to sort unrecognized values: ${util.stringify(a)} (typeof ${typeof a}) <-> ${util.stringify(b)} (typeof ${typeof b})`);
    }
    function sortSet(set) {
        const seen = new Set();
        return new Set(Array.from(set).filter((value) => checkForDuplicates('BTreeSet', seen, value)).sort(sortAsc));
    }
    function sortMap(map) {
        const seen = new Set();
        return new Map(Array.from(map.entries()).filter(([key]) => checkForDuplicates('BTreeMap', seen, key)).sort(([keyA], [keyB]) => sortAsc(keyA, keyB)));
    }

    function typeToConstructor(registry, type) {
        return typeof type === 'function'
            ? type
            : registry.createClassUnsafe(type);
    }
    function typesToConstructors(registry, types) {
        const count = types.length;
        const result = new Array(count);
        for (let i = 0; i < count; i++) {
            result[i] = typeToConstructor(registry, types[i]);
        }
        return result;
    }
    function mapToTypeMap(registry, input) {
        const entries = Object.entries(input);
        const count = entries.length;
        const output = [new Array(count), new Array(count)];
        for (let i = 0; i < count; i++) {
            output[1][i] = entries[i][0];
            output[0][i] = typeToConstructor(registry, entries[i][1]);
        }
        return output;
    }

    function typesToMap(registry, [Types, keys]) {
        const result = {};
        for (let i = 0, count = keys.length; i < count; i++) {
            result[keys[i]] = registry.getClassName(Types[i]) || new Types[i](registry).toRawType();
        }
        return result;
    }

    function decodeCompact(registry, Type, value) {
        if (util.isU8a(value)) {
            const [decodedLength, bn] = (value[0] & 0b11) < 0b11
                ? util.compactFromU8aLim(value)
                : util.compactFromU8a(value);
            return [new Type(registry, bn), decodedLength];
        }
        else if (value instanceof Compact) {
            const raw = value.unwrap();
            return raw instanceof Type
                ? [raw, 0]
                : [new Type(registry, raw), 0];
        }
        else if (value instanceof Type) {
            return [value, 0];
        }
        return [new Type(registry, value), 0];
    }
    class Compact {
        registry;
        createdAtHash;
        initialU8aLength;
        isStorageFallback;
        #Type;
        #raw;
        constructor(registry, Type, value = 0, { definition, setDefinition = util.identity } = {}) {
            this.registry = registry;
            this.#Type = definition || setDefinition(typeToConstructor(registry, Type));
            const [raw, decodedLength] = decodeCompact(registry, this.#Type, value);
            this.initialU8aLength = decodedLength;
            this.#raw = raw;
        }
        static with(Type) {
            let definition;
            const setDefinition = (d) => (definition = d);
            return class extends Compact {
                constructor(registry, value) {
                    super(registry, Type, value, { definition, setDefinition });
                }
            };
        }
        get encodedLength() {
            return this.toU8a().length;
        }
        get hash() {
            return this.registry.hash(this.toU8a());
        }
        get isEmpty() {
            return this.#raw.isEmpty;
        }
        bitLength() {
            return this.#raw.bitLength();
        }
        eq(other) {
            return this.#raw.eq(other instanceof Compact
                ? other.#raw
                : other);
        }
        inspect() {
            return {
                outer: [this.toU8a()]
            };
        }
        toBigInt() {
            return this.#raw.toBigInt();
        }
        toBn() {
            return this.#raw.toBn();
        }
        toHex(isLe) {
            return this.#raw.toHex(isLe);
        }
        toHuman(isExtended, disableAscii) {
            return this.#raw.toHuman(isExtended, disableAscii);
        }
        toJSON() {
            return this.#raw.toJSON();
        }
        toNumber() {
            return this.#raw.toNumber();
        }
        toPrimitive(disableAscii) {
            return this.#raw.toPrimitive(disableAscii);
        }
        toRawType() {
            return `Compact<${this.registry.getClassName(this.#Type) || this.#raw.toRawType()}>`;
        }
        toString() {
            return this.#raw.toString();
        }
        toU8a(_isBare) {
            return util.compactToU8a(this.#raw.toBn());
        }
        unwrap() {
            return this.#raw;
        }
    }

    class DoNotConstruct {
        registry;
        createdAtHash;
        isStorageFallback;
        #neverError;
        constructor(registry, typeName = 'DoNotConstruct') {
            this.registry = registry;
            this.#neverError = new Error(`DoNotConstruct: Cannot construct unknown type ${typeName}`);
            throw this.#neverError;
        }
        static with(typeName) {
            return class extends DoNotConstruct {
                constructor(registry) {
                    super(registry, typeName);
                }
            };
        }
        get encodedLength() {
            throw this.#neverError;
        }
        get hash() {
            throw this.#neverError;
        }
        get isEmpty() {
            throw this.#neverError;
        }
        eq() {
            throw this.#neverError;
        }
        inspect() {
            throw this.#neverError;
        }
        toHex() {
            throw this.#neverError;
        }
        toHuman() {
            throw this.#neverError;
        }
        toJSON() {
            throw this.#neverError;
        }
        toPrimitive() {
            throw this.#neverError;
        }
        toRawType() {
            throw this.#neverError;
        }
        toString() {
            throw this.#neverError;
        }
        toU8a() {
            throw this.#neverError;
        }
    }

    class Null {
        encodedLength = 0;
        isEmpty = true;
        registry;
        createdAtHash;
        initialU8aLength = 0;
        isStorageFallback;
        constructor(registry) {
            this.registry = registry;
        }
        get hash() {
            throw new Error('.hash is not implemented on Null');
        }
        eq(other) {
            return other instanceof Null || util.isNull(other);
        }
        inspect() {
            return {};
        }
        toHex() {
            return '0x';
        }
        toHuman() {
            return this.toJSON();
        }
        toJSON() {
            return null;
        }
        toPrimitive() {
            return null;
        }
        toRawType() {
            return 'Null';
        }
        toString() {
            return '';
        }
        toU8a(_isBare) {
            return new Uint8Array();
        }
    }

    function isRustEnum$1(def) {
        const defValues = Object.values(def);
        if (defValues.some((v) => util.isNumber(v))) {
            if (!defValues.every((v) => util.isNumber(v) && v >= 0 && v <= 255)) {
                throw new Error('Invalid number-indexed enum definition');
            }
            return false;
        }
        return true;
    }
    function extractDef(registry, _def) {
        const def = {};
        let isBasic;
        let isIndexed;
        if (Array.isArray(_def)) {
            for (let i = 0, count = _def.length; i < count; i++) {
                def[_def[i]] = { Type: Null, index: i };
            }
            isBasic = true;
            isIndexed = false;
        }
        else if (isRustEnum$1(_def)) {
            const [Types, keys] = mapToTypeMap(registry, _def);
            for (let i = 0, count = keys.length; i < count; i++) {
                def[keys[i]] = { Type: Types[i], index: i };
            }
            isBasic = !Object.values(def).some(({ Type }) => Type !== Null);
            isIndexed = false;
        }
        else {
            const entries = Object.entries(_def);
            for (let i = 0, count = entries.length; i < count; i++) {
                const [key, index] = entries[i];
                def[key] = { Type: Null, index };
            }
            isBasic = true;
            isIndexed = true;
        }
        return {
            def,
            isBasic,
            isIndexed
        };
    }
    function getEntryType(def, checkIdx) {
        const values = Object.values(def);
        for (let i = 0, count = values.length; i < count; i++) {
            const { Type, index } = values[i];
            if (index === checkIdx) {
                return Type;
            }
        }
        throw new Error(`Unable to create Enum via index ${checkIdx}, in ${Object.keys(def).join(', ')}`);
    }
    function createFromU8a(registry, def, index, value) {
        const Type = getEntryType(def, index);
        return {
            index,
            value: new Type(registry, value)
        };
    }
    function createFromValue(registry, def, index = 0, value) {
        const Type = getEntryType(def, index);
        return {
            index,
            value: value instanceof Type
                ? value
                : new Type(registry, value)
        };
    }
    function decodeFromJSON(registry, def, key, value) {
        const keys = Object.keys(def).map((k) => k.toLowerCase());
        const keyLower = key.toLowerCase();
        const index = keys.indexOf(keyLower);
        if (index === -1) {
            throw new Error(`Cannot map Enum JSON, unable to find '${key}' in ${keys.join(', ')}`);
        }
        try {
            return createFromValue(registry, def, Object.values(def)[index].index, value);
        }
        catch (error) {
            throw new Error(`Enum(${key}):: ${error.message}`);
        }
    }
    function decodeEnum(registry, def, value, index) {
        if (util.isNumber(index)) {
            return createFromValue(registry, def, index, value);
        }
        else if (util.isU8a(value) || util.isHex(value)) {
            const u8a = util.u8aToU8a(value);
            if (u8a.length) {
                return createFromU8a(registry, def, u8a[0], u8a.subarray(1));
            }
        }
        else if (value instanceof Enum) {
            return createFromValue(registry, def, value.index, value.value);
        }
        else if (util.isNumber(value)) {
            return createFromValue(registry, def, value);
        }
        else if (util.isString(value)) {
            return decodeFromJSON(registry, def, value.toString());
        }
        else if (util.isObject(value)) {
            const key = Object.keys(value)[0];
            return decodeFromJSON(registry, def, key, value[key]);
        }
        return createFromValue(registry, def, Object.values(def)[0].index);
    }
    class Enum {
        registry;
        createdAtHash;
        initialU8aLength;
        isStorageFallback;
        #def;
        #entryIndex;
        #indexes;
        #isBasic;
        #isIndexed;
        #raw;
        constructor(registry, Types, value, index, { definition, setDefinition = util.identity } = {}) {
            const { def, isBasic, isIndexed } = definition || setDefinition(extractDef(registry, Types));
            const decoded = util.isU8a(value) && value.length && !util.isNumber(index)
                ? createFromU8a(registry, def, value[0], value.subarray(1))
                : decodeEnum(registry, def, value, index);
            this.registry = registry;
            this.#def = def;
            this.#isBasic = isBasic;
            this.#isIndexed = isIndexed;
            this.#indexes = Object.values(def).map(({ index }) => index);
            this.#entryIndex = this.#indexes.indexOf(decoded.index);
            this.#raw = decoded.value;
            if (this.#raw.initialU8aLength) {
                this.initialU8aLength = 1 + this.#raw.initialU8aLength;
            }
        }
        static with(Types) {
            let definition;
            const setDefinition = (d) => definition = d;
            return class extends Enum {
                static {
                    const keys = Array.isArray(Types)
                        ? Types
                        : Object.keys(Types);
                    const count = keys.length;
                    const asKeys = new Array(count);
                    const isKeys = new Array(count);
                    for (let i = 0; i < count; i++) {
                        const name = util.stringPascalCase(keys[i]);
                        asKeys[i] = `as${name}`;
                        isKeys[i] = `is${name}`;
                    }
                    util.objectProperties(this.prototype, isKeys, (_, i, self) => self.type === keys[i]);
                    util.objectProperties(this.prototype, asKeys, (k, i, self) => {
                        if (self.type !== keys[i]) {
                            throw new Error(`Cannot convert '${self.type}' via ${k}`);
                        }
                        return self.value;
                    });
                }
                constructor(registry, value, index) {
                    super(registry, Types, value, index, { definition, setDefinition });
                }
            };
        }
        get encodedLength() {
            return 1 + this.#raw.encodedLength;
        }
        get hash() {
            return this.registry.hash(this.toU8a());
        }
        get index() {
            return this.#indexes[this.#entryIndex];
        }
        get inner() {
            return this.#raw;
        }
        get isBasic() {
            return this.#isBasic;
        }
        get isEmpty() {
            return this.#raw.isEmpty;
        }
        get isNone() {
            return this.#raw instanceof Null;
        }
        get defIndexes() {
            return this.#indexes;
        }
        get defKeys() {
            return Object.keys(this.#def);
        }
        get type() {
            return this.defKeys[this.#entryIndex];
        }
        get value() {
            return this.#raw;
        }
        eq(other) {
            if (util.isU8a(other)) {
                return !this.toU8a().some((entry, index) => entry !== other[index]);
            }
            else if (util.isNumber(other)) {
                return this.toNumber() === other;
            }
            else if (this.#isBasic && util.isString(other)) {
                return this.type === other;
            }
            else if (util.isHex(other)) {
                return this.toHex() === other;
            }
            else if (other instanceof Enum) {
                return this.index === other.index && this.value.eq(other.value);
            }
            else if (util.isObject(other)) {
                return this.value.eq(other[this.type]);
            }
            return this.value.eq(other);
        }
        inspect() {
            if (this.#isBasic) {
                return { outer: [new Uint8Array([this.index])] };
            }
            const { inner, outer = [] } = this.#raw.inspect();
            return {
                inner,
                outer: [new Uint8Array([this.index]), ...outer]
            };
        }
        toHex() {
            return util.u8aToHex(this.toU8a());
        }
        toHuman(isExtended, disableAscii) {
            return this.#isBasic || this.isNone
                ? this.type
                : { [this.type]: this.#raw.toHuman(isExtended, disableAscii) };
        }
        toJSON() {
            return this.#isBasic
                ? this.type
                : { [util.stringCamelCase(this.type)]: this.#raw.toJSON() };
        }
        toNumber() {
            return this.index;
        }
        toPrimitive(disableAscii) {
            return this.#isBasic
                ? this.type
                : { [util.stringCamelCase(this.type)]: this.#raw.toPrimitive(disableAscii) };
        }
        _toRawStruct() {
            if (this.#isBasic) {
                return this.#isIndexed
                    ? this.defKeys.reduce((out, key, index) => {
                        out[key] = this.#indexes[index];
                        return out;
                    }, {})
                    : this.defKeys;
            }
            const entries = Object.entries(this.#def);
            return typesToMap(this.registry, entries.reduce((out, [key, { Type }], i) => {
                out[0][i] = Type;
                out[1][i] = key;
                return out;
            }, [new Array(entries.length), new Array(entries.length)]));
        }
        toRawType() {
            return util.stringify({ _enum: this._toRawStruct() });
        }
        toString() {
            return this.isNone
                ? this.type
                : util.stringify(this.toJSON());
        }
        toU8a(isBare) {
            return isBare
                ? this.#raw.toU8a(isBare)
                : util.u8aConcatStrict([
                    new Uint8Array([this.index]),
                    this.#raw.toU8a(isBare)
                ]);
        }
    }

    class Int extends AbstractInt {
        constructor(registry, value = 0, bitLength) {
            super(registry, value, bitLength, true);
        }
        static with(bitLength, typeName) {
            return class extends Int {
                constructor(registry, value) {
                    super(registry, value, bitLength);
                }
                toRawType() {
                    return typeName || super.toRawType();
                }
            };
        }
    }

    class None extends Null {
        toRawType() {
            return 'None';
        }
    }
    function decodeOption(registry, Type, value) {
        if (value instanceof Type) {
            return value;
        }
        else if (value instanceof Option) {
            if (value.value instanceof Type) {
                return value.value;
            }
            else if (value.isNone) {
                return new None(registry);
            }
            return new Type(registry, value.value);
        }
        else if (util.isNull(value) || util.isUndefined(value) || value === '0x' || value instanceof None) {
            return new None(registry);
        }
        else if (util.isU8a(value)) {
            return !value.length || value[0] === 0
                ? new None(registry)
                : new Type(registry, value.subarray(1));
        }
        return new Type(registry, value);
    }
    class Option {
        registry;
        createdAtHash;
        initialU8aLength;
        isStorageFallback;
        #Type;
        #raw;
        constructor(registry, typeName, value, { definition, setDefinition = util.identity } = {}) {
            const Type = definition || setDefinition(typeToConstructor(registry, typeName));
            const decoded = util.isU8a(value) && value.length && !util.isCodec(value)
                ? value[0] === 0
                    ? new None(registry)
                    : new Type(registry, value.subarray(1))
                : decodeOption(registry, Type, value);
            this.registry = registry;
            this.#Type = Type;
            this.#raw = decoded;
            if (decoded?.initialU8aLength) {
                this.initialU8aLength = 1 + decoded.initialU8aLength;
            }
        }
        static with(Type) {
            let definition;
            const setDefinition = (d) => {
                definition = d;
                return d;
            };
            return class extends Option {
                constructor(registry, value) {
                    super(registry, Type, value, { definition, setDefinition });
                }
            };
        }
        get encodedLength() {
            return 1 + this.#raw.encodedLength;
        }
        get hash() {
            return this.registry.hash(this.toU8a());
        }
        get isEmpty() {
            return this.isNone;
        }
        get isNone() {
            return this.#raw instanceof None;
        }
        get isSome() {
            return !this.isNone;
        }
        get value() {
            return this.#raw;
        }
        eq(other) {
            if (other instanceof Option) {
                return (this.isSome === other.isSome) && this.value.eq(other.value);
            }
            return this.value.eq(other);
        }
        inspect() {
            if (this.isNone) {
                return { outer: [new Uint8Array([0])] };
            }
            const { inner, outer = [] } = this.#raw.inspect();
            return {
                inner,
                outer: [new Uint8Array([1]), ...outer]
            };
        }
        toHex() {
            return this.isNone
                ? '0x'
                : util.u8aToHex(this.toU8a().subarray(1));
        }
        toHuman(isExtended, disableAscii) {
            return this.#raw.toHuman(isExtended, disableAscii);
        }
        toJSON() {
            return this.isNone
                ? null
                : this.#raw.toJSON();
        }
        toPrimitive(disableAscii) {
            return this.isNone
                ? null
                : this.#raw.toPrimitive(disableAscii);
        }
        toRawType(isBare) {
            const wrapped = this.registry.getClassName(this.#Type) || new this.#Type(this.registry).toRawType();
            return isBare
                ? wrapped
                : `Option<${wrapped}>`;
        }
        toString() {
            return this.#raw.toString();
        }
        toU8a(isBare) {
            if (isBare) {
                return this.#raw.toU8a(true);
            }
            const u8a = new Uint8Array(this.encodedLength);
            if (this.isSome) {
                u8a.set([1]);
                u8a.set(this.#raw.toU8a(), 1);
            }
            return u8a;
        }
        unwrap() {
            if (this.isNone) {
                throw new Error('Option: unwrapping a None value');
            }
            return this.#raw;
        }
        unwrapOr(defaultValue) {
            return this.isSome
                ? this.unwrap()
                : defaultValue;
        }
        unwrapOrDefault() {
            return this.isSome
                ? this.unwrap()
                : new this.#Type(this.registry);
        }
    }

    class Result extends Enum {
        constructor(registry, Ok, Err, value) {
            super(registry, { Ok, Err }, value);
        }
        static with(Types) {
            return class extends Result {
                constructor(registry, value) {
                    super(registry, Types.Ok, Types.Err, value);
                }
            };
        }
        get asErr() {
            if (!this.isErr) {
                throw new Error('Cannot extract Err value from Ok result, check isErr first');
            }
            return this.value;
        }
        get asOk() {
            if (!this.isOk) {
                throw new Error('Cannot extract Ok value from Err result, check isOk first');
            }
            return this.value;
        }
        get isEmpty() {
            return this.isOk && this.value.isEmpty;
        }
        get isErr() {
            return !this.isOk;
        }
        get isOk() {
            return this.index === 0;
        }
        toRawType() {
            const Types = this._toRawStruct();
            return `Result<${Types.Ok},${Types.Err}>`;
        }
    }

    function decodeTuple(registry, result, value, Classes) {
        if (Array.isArray(value)) {
            const Types = Classes[0];
            for (let i = 0, count = Types.length; i < count; i++) {
                try {
                    const entry = value?.[i];
                    result[i] = entry instanceof Types[i]
                        ? entry
                        : new Types[i](registry, entry);
                }
                catch (error) {
                    throw new Error(`Tuple: failed on ${i}:: ${error.message}`);
                }
            }
            return [result, 0];
        }
        else if (util.isHex(value)) {
            return decodeU8a$7(registry, result, util.u8aToU8a(value), Classes);
        }
        else if (!value || !result.length) {
            const Types = Classes[0];
            for (let i = 0, count = Types.length; i < count; i++) {
                result[i] = new Types[i](registry);
            }
            return [result, 0];
        }
        throw new Error(`Expected array input to Tuple decoding, found ${typeof value}: ${util.stringify(value)}`);
    }
    class Tuple extends AbstractArray {
        #Types;
        constructor(registry, Types, value, { definition, setDefinition = util.identity } = {}) {
            const Classes = definition || setDefinition(Array.isArray(Types)
                ? [typesToConstructors(registry, Types), []]
                : util.isFunction(Types) || util.isString(Types)
                    ? [[typeToConstructor(registry, Types)], []]
                    : mapToTypeMap(registry, Types));
            super(registry, Classes[0].length);
            this.initialU8aLength = (util.isU8a(value)
                ? decodeU8a$7(registry, this, value, Classes)
                : decodeTuple(registry, this, value, Classes))[1];
            this.#Types = Classes;
        }
        static with(Types) {
            let definition;
            const setDefinition = (d) => definition = d;
            return class extends Tuple {
                constructor(registry, value) {
                    super(registry, Types, value, { definition, setDefinition });
                }
            };
        }
        get encodedLength() {
            let total = 0;
            for (let i = 0, count = this.length; i < count; i++) {
                total += this[i].encodedLength;
            }
            return total;
        }
        get Types() {
            return this.#Types[1].length
                ? this.#Types[1]
                : this.#Types[0].map((T) => new T(this.registry).toRawType());
        }
        inspect() {
            return {
                inner: this.inspectInner()
            };
        }
        toRawType() {
            const types = this.#Types[0].map((T) => this.registry.getClassName(T) || new T(this.registry).toRawType());
            return `(${types.join(',')})`;
        }
        toString() {
            return util.stringify(this.toJSON());
        }
        toU8a(isBare) {
            return util.u8aConcatStrict(this.toU8aInner(isBare));
        }
    }

    class UInt extends AbstractInt {
        static with(bitLength, typeName) {
            return class extends UInt {
                constructor(registry, value) {
                    super(registry, value, bitLength);
                }
                toRawType() {
                    return typeName || super.toRawType();
                }
            };
        }
    }

    const MAX_LENGTH$2 = 512 * 1024;
    const l$5 = util.logger('Vec');
    function decodeVecLength(value) {
        if (Array.isArray(value)) {
            return [value, value.length, 0];
        }
        else if (util.isU8a(value) || util.isHex(value)) {
            const u8a = util.u8aToU8a(value);
            const [startAt, length] = util.compactFromU8aLim(u8a);
            if (length > MAX_LENGTH$2) {
                throw new Error(`Vec length ${length.toString()} exceeds ${MAX_LENGTH$2}`);
            }
            return [u8a, length, startAt];
        }
        else if (!value) {
            return [null, 0, 0];
        }
        throw new Error(`Expected array/hex input to Vec<*> decoding, found ${typeof value}: ${util.stringify(value)}`);
    }
    function decodeVec(registry, result, value, startAt, Type) {
        if (Array.isArray(value)) {
            const count = result.length;
            for (let i = 0; i < count; i++) {
                const entry = value[i];
                try {
                    result[i] = entry instanceof Type
                        ? entry
                        : new Type(registry, entry);
                }
                catch (error) {
                    l$5.error(`Unable to decode on index ${i}`, error.message);
                    throw error;
                }
            }
            return [0, 0];
        }
        else if (!value) {
            return [0, 0];
        }
        return decodeU8aVec(registry, result, util.u8aToU8a(value), startAt, Type);
    }
    class Vec extends AbstractArray {
        #Type;
        constructor(registry, Type, value = [], { definition, setDefinition = util.identity } = {}) {
            const [decodeFrom, length, startAt] = decodeVecLength(value);
            super(registry, length);
            this.#Type = definition || setDefinition(typeToConstructor(registry, Type));
            this.initialU8aLength = (util.isU8a(decodeFrom)
                ? decodeU8aVec(registry, this, decodeFrom, startAt, this.#Type)
                : decodeVec(registry, this, decodeFrom, startAt, this.#Type))[0];
        }
        static with(Type) {
            let definition;
            const setDefinition = (d) => (definition = d);
            return class extends Vec {
                constructor(registry, value) {
                    super(registry, Type, value, { definition, setDefinition });
                }
            };
        }
        get Type() {
            return this.#Type.name;
        }
        indexOf(other) {
            const check = other instanceof this.#Type
                ? other
                : new this.#Type(this.registry, other);
            for (let i = 0, count = this.length; i < count; i++) {
                if (check.eq(this[i])) {
                    return i;
                }
            }
            return -1;
        }
        toRawType() {
            return `Vec<${this.registry.getClassName(this.#Type) || new this.#Type(this.registry).toRawType()}>`;
        }
    }

    class VecFixed extends AbstractArray {
        #Type;
        constructor(registry, Type, length, value = [], { definition, setDefinition = util.identity } = {}) {
            super(registry, length);
            this.#Type = definition || setDefinition(typeToConstructor(registry, Type));
            this.initialU8aLength = (util.isU8a(value)
                ? decodeU8aVec(registry, this, value, 0, this.#Type)
                : decodeVec(registry, this, value, 0, this.#Type))[1];
        }
        static with(Type, length) {
            let definition;
            const setDefinition = (d) => (definition = d);
            return class extends VecFixed {
                constructor(registry, value) {
                    super(registry, Type, length, value, { definition, setDefinition });
                }
            };
        }
        get Type() {
            return new this.#Type(this.registry).toRawType();
        }
        get encodedLength() {
            let total = 0;
            for (let i = 0, count = this.length; i < count; i++) {
                total += this[i].encodedLength;
            }
            return total;
        }
        inspect() {
            return {
                inner: this.inspectInner()
            };
        }
        toU8a() {
            const encoded = this.toU8aInner();
            return encoded.length
                ? util.u8aConcatStrict(encoded)
                : new Uint8Array([]);
        }
        toRawType() {
            return `[${this.Type};${this.length}]`;
        }
    }

    class Raw extends Uint8Array {
        registry;
        createdAtHash;
        initialU8aLength;
        isStorageFallback;
        static get [Symbol.species]() {
            return Uint8Array;
        }
        constructor(registry, value, initialU8aLength) {
            super(util.u8aToU8a(value));
            this.registry = registry;
            this.initialU8aLength = initialU8aLength;
        }
        get encodedLength() {
            return this.length;
        }
        get hash() {
            return this.registry.hash(this.toU8a());
        }
        get isAscii() {
            return util.isAscii(this);
        }
        get isEmpty() {
            return !this.length || util.isUndefined(this.find((b) => !!b));
        }
        get isUtf8() {
            return util.isUtf8(this);
        }
        bitLength() {
            return this.length * 8;
        }
        eq(other) {
            if (other instanceof Uint8Array) {
                return (this.length === other.length) &&
                    !this.some((b, index) => b !== other[index]);
            }
            return this.eq(util.u8aToU8a(other));
        }
        inspect() {
            return {
                outer: [this.toU8a()]
            };
        }
        toHex() {
            return util.u8aToHex(this);
        }
        toHuman(_isExtended, disableAscii) {
            return this.toPrimitive(disableAscii);
        }
        toJSON() {
            return this.toHex();
        }
        toPrimitive(disableAscii) {
            if (!disableAscii && this.isAscii) {
                const text = this.toUtf8();
                if (util.isAscii(text)) {
                    return text;
                }
            }
            return this.toJSON();
        }
        toRawType() {
            return 'Raw';
        }
        toString() {
            return this.toHex();
        }
        toU8a(_isBare) {
            return Uint8Array.from(this);
        }
        toUtf8() {
            if (!this.isUtf8) {
                throw new Error('The character sequence is not a valid Utf8 string');
            }
            return util.u8aToString(this);
        }
    }

    function decodeBitVecU8a(value) {
        if (!value?.length) {
            return [0, new Uint8Array()];
        }
        const [offset, length] = util.compactFromU8aLim(value);
        const total = offset + Math.ceil(length / 8);
        if (total > value.length) {
            throw new Error(`BitVec: required length less than remainder, expected at least ${total}, found ${value.length}`);
        }
        return [length, value.subarray(offset, total)];
    }
    function decodeBitVec(value) {
        if (Array.isArray(value) || util.isString(value)) {
            const u8a = util.u8aToU8a(value);
            return [u8a.length * 8, u8a];
        }
        return decodeBitVecU8a(value);
    }
    class BitVec extends Raw {
        #decodedLength;
        #isMsb;
        constructor(registry, value, isMsb = false) {
            const [decodedLength, u8a] = decodeBitVec(value);
            super(registry, u8a);
            this.#decodedLength = decodedLength;
            this.#isMsb = isMsb;
        }
        get encodedLength() {
            return this.length + util.compactToU8a(this.#decodedLength).length;
        }
        inspect() {
            return {
                outer: [util.compactToU8a(this.#decodedLength), super.toU8a()]
            };
        }
        toBoolArray() {
            const map = [...this.toU8a(true)].map((v) => [
                !!(v & 0b1000_0000),
                !!(v & 0b0100_0000),
                !!(v & 0b0010_0000),
                !!(v & 0b0001_0000),
                !!(v & 0b0000_1000),
                !!(v & 0b0000_0100),
                !!(v & 0b0000_0010),
                !!(v & 0b0000_0001)
            ]);
            const count = map.length;
            const result = new Array(8 * count);
            for (let i = 0; i < count; i++) {
                const off = i * 8;
                const v = map[i];
                for (let j = 0; j < 8; j++) {
                    result[off + j] = this.#isMsb
                        ? v[j]
                        : v[7 - j];
                }
            }
            return result;
        }
        toHuman() {
            return `0b${[...this.toU8a(true)]
            .map((d) => `00000000${d.toString(2)}`.slice(-8))
            .map((s) => this.#isMsb ? s : s.split('').reverse().join(''))
            .join('_')}`;
        }
        toRawType() {
            return 'BitVec';
        }
        toU8a(isBare) {
            const bitVec = super.toU8a(isBare);
            return isBare
                ? bitVec
                : util.u8aConcatStrict([util.compactToU8a(this.#decodedLength), bitVec]);
        }
    }

    function noopSetDefinition(d) {
        return d;
    }
    function decodeStructFromObject(registry, [Types, keys], value, jsonMap) {
        let jsonObj;
        const typeofArray = Array.isArray(value);
        const typeofMap = value instanceof Map;
        const count = keys.length;
        if (!typeofArray && !typeofMap && !util.isObject(value)) {
            throw new Error(`Struct: Cannot decode value ${util.stringify(value)} (typeof ${typeof value}), expected an input object, map or array`);
        }
        else if (typeofArray && value.length !== count) {
            throw new Error(`Struct: Unable to map ${util.stringify(value)} array to object with known keys ${keys.join(', ')}`);
        }
        const raw = new Array(count);
        for (let i = 0; i < count; i++) {
            const key = keys[i];
            const jsonKey = jsonMap.get(key) || key;
            const Type = Types[i];
            let assign;
            try {
                if (typeofArray) {
                    assign = value[i];
                }
                else if (typeofMap) {
                    assign = jsonKey && value.get(jsonKey);
                }
                else {
                    assign = jsonKey && Object.prototype.hasOwnProperty.call(value, jsonKey) ? value[jsonKey] : undefined;
                    if (util.isUndefined(assign)) {
                        if (util.isUndefined(jsonObj)) {
                            const entries = Object.entries(value);
                            jsonObj = {};
                            for (let e = 0, ecount = entries.length; e < ecount; e++) {
                                if (Object.prototype.hasOwnProperty.call(value, entries[e][0])) {
                                    jsonObj[util.stringCamelCase(entries[e][0])] = entries[e][1];
                                }
                            }
                        }
                        assign = jsonKey && Object.prototype.hasOwnProperty.call(jsonObj, jsonKey) ? jsonObj[jsonKey] : undefined;
                    }
                }
                raw[i] = [
                    key,
                    assign instanceof Type
                        ? assign
                        : new Type(registry, assign)
                ];
            }
            catch (error) {
                let type = Type.name;
                try {
                    type = new Type(registry).toRawType();
                }
                catch {
                }
                throw new Error(`Struct: failed on ${jsonKey}: ${type}:: ${error.message}`);
            }
        }
        return [raw, 0];
    }
    class Struct extends Map {
        registry;
        createdAtHash;
        initialU8aLength;
        isStorageFallback;
        #jsonMap;
        #Types;
        constructor(registry, Types, value, jsonMap = new Map(), { definition, setDefinition = noopSetDefinition } = {}) {
            const typeMap = definition || setDefinition(mapToTypeMap(registry, Types));
            const [decoded, decodedLength] = util.isU8a(value) || util.isHex(value)
                ? decodeU8aStruct(registry, new Array(typeMap[0].length), util.u8aToU8a(value), typeMap)
                : value instanceof Struct
                    ? [value, 0]
                    : decodeStructFromObject(registry, typeMap, value || {}, jsonMap);
            super(decoded);
            this.initialU8aLength = decodedLength;
            this.registry = registry;
            this.#jsonMap = jsonMap;
            this.#Types = typeMap;
        }
        static with(Types, jsonMap) {
            let definition;
            const setDefinition = (d) => definition = d;
            return class extends Struct {
                static {
                    const keys = Object.keys(Types);
                    util.objectProperties(this.prototype, keys, (k, _, self) => self.get(k));
                }
                constructor(registry, value) {
                    super(registry, Types, value, jsonMap, { definition, setDefinition });
                }
            };
        }
        get defKeys() {
            return this.#Types[1];
        }
        get isEmpty() {
            return [...this.keys()].length === 0;
        }
        get encodedLength() {
            let total = 0;
            for (const v of this.values()) {
                total += v.encodedLength;
            }
            return total;
        }
        get hash() {
            return this.registry.hash(this.toU8a());
        }
        get Type() {
            const result = {};
            const [Types, keys] = this.#Types;
            for (let i = 0, count = keys.length; i < count; i++) {
                result[keys[i]] = new Types[i](this.registry).toRawType();
            }
            return result;
        }
        eq(other) {
            return compareMap(this, other);
        }
        get(key) {
            return super.get(key);
        }
        getAtIndex(index) {
            return this.toArray()[index];
        }
        getT(key) {
            return super.get(key);
        }
        inspect(isBare) {
            const inner = [];
            for (const [k, v] of this.entries()) {
                inner.push({
                    ...v.inspect(!isBare || util.isBoolean(isBare)
                        ? isBare
                        : isBare[k]),
                    name: util.stringCamelCase(k)
                });
            }
            return {
                inner
            };
        }
        toArray() {
            return [...this.values()];
        }
        toHex() {
            return util.u8aToHex(this.toU8a());
        }
        toHuman(isExtended, disableAscii) {
            const json = {};
            for (const [k, v] of this.entries()) {
                json[k] = v.toHuman(isExtended, disableAscii);
            }
            return json;
        }
        toJSON() {
            const json = {};
            for (const [k, v] of this.entries()) {
                json[(this.#jsonMap.get(k) || k)] = v.toJSON();
            }
            return json;
        }
        toPrimitive(disableAscii) {
            const json = {};
            for (const [k, v] of this.entries()) {
                json[k] = v.toPrimitive(disableAscii);
            }
            return json;
        }
        toRawType() {
            return util.stringify(typesToMap(this.registry, this.#Types));
        }
        toString() {
            return util.stringify(this.toJSON());
        }
        toU8a(isBare) {
            const encoded = [];
            for (const [k, v] of this.entries()) {
                encoded.push(v.toU8a(!isBare || util.isBoolean(isBare)
                    ? isBare
                    : isBare[k]));
            }
            return util.u8aConcatStrict(encoded);
        }
    }

    const l$4 = util.logger('Map');
    function decodeMapFromU8a(registry, KeyClass, ValClass, u8a) {
        const output = new Map();
        const [offset, count] = util.compactFromU8aLim(u8a);
        const types = [];
        for (let i = 0; i < count; i++) {
            types.push(KeyClass, ValClass);
        }
        const [values, decodedLength] = decodeU8a$7(registry, new Array(types.length), u8a.subarray(offset), [types, []]);
        for (let i = 0, count = values.length; i < count; i += 2) {
            output.set(values[i], values[i + 1]);
        }
        return [KeyClass, ValClass, output, offset + decodedLength];
    }
    function decodeMapFromMap(registry, KeyClass, ValClass, value) {
        const output = new Map();
        for (const [key, val] of value.entries()) {
            const isComplex = KeyClass.prototype instanceof AbstractArray ||
                KeyClass.prototype instanceof Struct ||
                KeyClass.prototype instanceof Enum;
            try {
                output.set(key instanceof KeyClass
                    ? key
                    : new KeyClass(registry, isComplex && typeof key === 'string' ? JSON.parse(key) : key), val instanceof ValClass
                    ? val
                    : new ValClass(registry, val));
            }
            catch (error) {
                l$4.error('Failed to decode key or value:', error.message);
                throw error;
            }
        }
        return [KeyClass, ValClass, output, 0];
    }
    function decodeMap(registry, keyType, valType, value) {
        const KeyClass = typeToConstructor(registry, keyType);
        const ValClass = typeToConstructor(registry, valType);
        if (!value) {
            return [KeyClass, ValClass, new Map(), 0];
        }
        else if (util.isU8a(value) || util.isHex(value)) {
            return decodeMapFromU8a(registry, KeyClass, ValClass, util.u8aToU8a(value));
        }
        else if (value instanceof Map) {
            return decodeMapFromMap(registry, KeyClass, ValClass, value);
        }
        else if (util.isObject(value)) {
            return decodeMapFromMap(registry, KeyClass, ValClass, new Map(Object.entries(value)));
        }
        throw new Error('Map: cannot decode type');
    }
    class CodecMap extends Map {
        registry;
        createdAtHash;
        initialU8aLength;
        isStorageFallback;
        #KeyClass;
        #ValClass;
        #type;
        constructor(registry, keyType, valType, rawValue, type = 'HashMap') {
            const [KeyClass, ValClass, decoded, decodedLength] = decodeMap(registry, keyType, valType, rawValue);
            super(type === 'BTreeMap' ? sortMap(decoded) : decoded);
            this.registry = registry;
            this.initialU8aLength = decodedLength;
            this.#KeyClass = KeyClass;
            this.#ValClass = ValClass;
            this.#type = type;
        }
        get encodedLength() {
            let len = util.compactToU8a(this.size).length;
            for (const [k, v] of this.entries()) {
                len += k.encodedLength + v.encodedLength;
            }
            return len;
        }
        get hash() {
            return this.registry.hash(this.toU8a());
        }
        get isEmpty() {
            return this.size === 0;
        }
        eq(other) {
            return compareMap(this, other);
        }
        inspect() {
            const inner = [];
            for (const [k, v] of this.entries()) {
                inner.push(k.inspect());
                inner.push(v.inspect());
            }
            return {
                inner,
                outer: [util.compactToU8a(this.size)]
            };
        }
        toHex() {
            return util.u8aToHex(this.toU8a());
        }
        toHuman(isExtended, disableAscii) {
            const json = {};
            for (const [k, v] of this.entries()) {
                json[k instanceof Raw && !disableAscii && k.isAscii
                    ? k.toUtf8()
                    : k.toString()] = v.toHuman(isExtended, disableAscii);
            }
            return json;
        }
        toJSON() {
            const json = {};
            for (const [k, v] of this.entries()) {
                json[k.toString()] = v.toJSON();
            }
            return json;
        }
        toPrimitive(disableAscii) {
            const json = {};
            for (const [k, v] of this.entries()) {
                json[k instanceof Raw && !disableAscii && k.isAscii
                    ? k.toUtf8()
                    : k.toString()] = v.toPrimitive(disableAscii);
            }
            return json;
        }
        toRawType() {
            return `${this.#type}<${this.registry.getClassName(this.#KeyClass) || new this.#KeyClass(this.registry).toRawType()},${this.registry.getClassName(this.#ValClass) || new this.#ValClass(this.registry).toRawType()}>`;
        }
        toString() {
            return util.stringify(this.toJSON());
        }
        toU8a(isBare) {
            const encoded = [];
            if (!isBare) {
                encoded.push(util.compactToU8a(this.size));
            }
            for (const [k, v] of this.entries()) {
                encoded.push(k.toU8a(isBare), v.toU8a(isBare));
            }
            return util.u8aConcatStrict(encoded);
        }
    }

    class BTreeMap extends CodecMap {
        static with(keyType, valType) {
            return class extends BTreeMap {
                constructor(registry, value) {
                    super(registry, keyType, valType, value, 'BTreeMap');
                }
            };
        }
    }

    const l$3 = util.logger('BTreeSet');
    function decodeSetFromU8a(registry, ValClass, u8a) {
        const output = new Set();
        const [offset, count] = util.compactFromU8aLim(u8a);
        const result = new Array(count);
        const [decodedLength] = decodeU8aVec(registry, result, u8a, offset, ValClass);
        for (let i = 0; i < count; i++) {
            output.add(result[i]);
        }
        return [ValClass, output, decodedLength];
    }
    function decodeSetFromSet(registry, ValClass, value) {
        const output = new Set();
        value.forEach((val) => {
            try {
                output.add((val instanceof ValClass) ? val : new ValClass(registry, val));
            }
            catch (error) {
                l$3.error('Failed to decode key or value:', error.message);
                throw error;
            }
        });
        return [ValClass, output, 0];
    }
    function decodeSet$1(registry, valType, value) {
        const ValClass = typeToConstructor(registry, valType);
        if (!value) {
            return [ValClass, new Set(), 0];
        }
        else if (util.isU8a(value) || util.isHex(value)) {
            return decodeSetFromU8a(registry, ValClass, util.u8aToU8a(value));
        }
        else if (Array.isArray(value) || value instanceof Set) {
            return decodeSetFromSet(registry, ValClass, value);
        }
        throw new Error('BTreeSet: cannot decode type');
    }
    class BTreeSet extends Set {
        registry;
        createdAtHash;
        initialU8aLength;
        isStorageFallback;
        #ValClass;
        constructor(registry, valType, rawValue) {
            const [ValClass, values, decodedLength] = decodeSet$1(registry, valType, rawValue);
            super(sortSet(values));
            this.registry = registry;
            this.initialU8aLength = decodedLength;
            this.#ValClass = ValClass;
        }
        static with(valType) {
            return class extends BTreeSet {
                constructor(registry, value) {
                    super(registry, valType, value);
                }
            };
        }
        get encodedLength() {
            let len = util.compactToU8a(this.size).length;
            for (const v of this.values()) {
                len += v.encodedLength;
            }
            return len;
        }
        get hash() {
            return this.registry.hash(this.toU8a());
        }
        get isEmpty() {
            return this.size === 0;
        }
        get strings() {
            return [...super.values()].map((v) => v.toString());
        }
        eq(other) {
            return compareSet(this, other);
        }
        inspect() {
            const inner = [];
            for (const v of this.values()) {
                inner.push(v.inspect());
            }
            return {
                inner,
                outer: [util.compactToU8a(this.size)]
            };
        }
        toHex() {
            return util.u8aToHex(this.toU8a());
        }
        toHuman(isExtended, disableAscii) {
            const json = [];
            for (const v of this.values()) {
                json.push(v.toHuman(isExtended, disableAscii));
            }
            return json;
        }
        toJSON() {
            const json = [];
            for (const v of this.values()) {
                json.push(v.toJSON());
            }
            return json;
        }
        toRawType() {
            return `BTreeSet<${this.registry.getClassName(this.#ValClass) || new this.#ValClass(this.registry).toRawType()}>`;
        }
        toPrimitive(disableAscii) {
            const json = [];
            for (const v of this.values()) {
                json.push(v.toPrimitive(disableAscii));
            }
            return json;
        }
        toString() {
            return util.stringify(this.toJSON());
        }
        toU8a(isBare) {
            const encoded = [];
            if (!isBare) {
                encoded.push(util.compactToU8a(this.size));
            }
            for (const v of this.values()) {
                encoded.push(v.toU8a(isBare));
            }
            return util.u8aConcatStrict(encoded);
        }
    }

    const MAX_LENGTH$1 = 10 * 1024 * 1024;
    function decodeBytesU8a(value) {
        if (!value.length) {
            return [new Uint8Array(), 0];
        }
        const [offset, length] = util.compactFromU8aLim(value);
        const total = offset + length;
        if (length > MAX_LENGTH$1) {
            throw new Error(`Bytes length ${length.toString()} exceeds ${MAX_LENGTH$1}`);
        }
        else if (total > value.length) {
            throw new Error(`Bytes: required length less than remainder, expected at least ${total}, found ${value.length}`);
        }
        return [value.subarray(offset, total), total];
    }
    class Bytes extends Raw {
        constructor(registry, value) {
            const [u8a, decodedLength] = util.isU8a(value) && !(value instanceof Raw)
                ? decodeBytesU8a(value)
                : Array.isArray(value) || util.isString(value)
                    ? [util.u8aToU8a(value), 0]
                    : [value, 0];
            super(registry, u8a, decodedLength);
        }
        get encodedLength() {
            return this.length + util.compactToU8a(this.length).length;
        }
        inspect(isBare) {
            const clength = util.compactToU8a(this.length);
            return {
                outer: isBare
                    ? [super.toU8a()]
                    : this.length
                        ? [clength, super.toU8a()]
                        : [clength]
            };
        }
        toRawType() {
            return 'Bytes';
        }
        toU8a(isBare) {
            return isBare
                ? super.toU8a(isBare)
                : util.compactAddLength(this);
        }
    }

    class HashMap extends CodecMap {
        static with(keyType, valType) {
            return class extends HashMap {
                constructor(registry, value) {
                    super(registry, keyType, valType, value);
                }
            };
        }
    }

    const EMPTY = new Uint8Array();
    class Linkage extends Struct {
        constructor(registry, Type, value) {
            super(registry, {
                previous: Option.with(Type),
                next: Option.with(Type)
            }, value);
        }
        static withKey(Type) {
            return class extends Linkage {
                constructor(registry, value) {
                    super(registry, Type, value);
                }
            };
        }
        get previous() {
            return this.get('previous');
        }
        get next() {
            return this.get('next');
        }
        toRawType() {
            return `Linkage<${this.next.toRawType(true)}>`;
        }
        toU8a(isBare) {
            return this.isEmpty
                ? EMPTY
                : super.toU8a(isBare);
        }
    }

    class bool extends Boolean {
        registry;
        createdAtHash;
        initialU8aLength = 1;
        isStorageFallback;
        constructor(registry, value = false) {
            super(util.isU8a(value)
                ? value[0] === 1
                : value instanceof Boolean
                    ? value.valueOf()
                    : !!value);
            this.registry = registry;
        }
        get encodedLength() {
            return 1 | 0;
        }
        get hash() {
            return this.registry.hash(this.toU8a());
        }
        get isEmpty() {
            return this.isFalse;
        }
        get isFalse() {
            return !this.isTrue;
        }
        get isTrue() {
            return this.valueOf();
        }
        eq(other) {
            return this.valueOf() === (other instanceof Boolean
                ? other.valueOf()
                : other);
        }
        inspect() {
            return {
                outer: [this.toU8a()]
            };
        }
        toHex() {
            return util.u8aToHex(this.toU8a());
        }
        toHuman() {
            return this.toJSON();
        }
        toJSON() {
            return this.valueOf();
        }
        toPrimitive() {
            return this.toJSON();
        }
        toRawType() {
            return 'bool';
        }
        toString() {
            return this.toJSON().toString();
        }
        toU8a(_isBare) {
            return new Uint8Array([this.valueOf() ? 1 : 0]);
        }
    }

    function decodeU8a$6(registry, value) {
        return value[0] === 0
            ? null
            : new bool(registry, value[0] === 1);
    }
    class OptionBool extends Option {
        constructor(registry, value) {
            super(registry, bool, util.isU8a(value) || util.isHex(value)
                ? decodeU8a$6(registry, util.u8aToU8a(value))
                : value);
            this.initialU8aLength = 1;
        }
        get encodedLength() {
            return 1 | 0;
        }
        get isFalse() {
            return this.isSome
                ? !this.value.valueOf()
                : false;
        }
        get isTrue() {
            return this.isSome
                ? this.value.valueOf()
                : false;
        }
        inspect() {
            return { outer: [this.toU8a()] };
        }
        toRawType(isBare) {
            return isBare
                ? 'bool'
                : 'Option<bool>';
        }
        toU8a(isBare) {
            if (isBare) {
                return super.toU8a(true);
            }
            return this.isSome
                ? new Uint8Array([this.isTrue ? 1 : 2])
                : new Uint8Array([0]);
        }
    }

    class Range extends Tuple {
        #rangeName;
        constructor(registry, Type, value, { rangeName = 'Range' } = {}) {
            super(registry, [Type, Type], value);
            this.#rangeName = rangeName;
        }
        static with(Type) {
            return class extends Range {
                constructor(registry, value) {
                    super(registry, Type, value);
                }
            };
        }
        get start() {
            return this[0];
        }
        get end() {
            return this[1];
        }
        toRawType() {
            return `${this.#rangeName}<${this.start.toRawType()}>`;
        }
    }

    class RangeInclusive extends Range {
        constructor(registry, Type, value) {
            super(registry, Type, value, { rangeName: 'RangeInclusive' });
        }
        static with(Type) {
            return class extends RangeInclusive {
                constructor(registry, value) {
                    super(registry, Type, value);
                }
            };
        }
    }

    const MAX_LENGTH = 128 * 1024;
    function decodeText(value) {
        if (util.isU8a(value)) {
            if (!value.length) {
                return ['', 0];
            }
            if (value instanceof Raw) {
                return [util.u8aToString(value), 0];
            }
            const [offset, length] = util.compactFromU8aLim(value);
            const total = offset + length;
            if (length > MAX_LENGTH) {
                throw new Error(`Text: length ${length.toString()} exceeds ${MAX_LENGTH}`);
            }
            else if (total > value.length) {
                throw new Error(`Text: required length less than remainder, expected at least ${total}, found ${value.length}`);
            }
            return [util.u8aToString(value.subarray(offset, total)), total];
        }
        else if (util.isHex(value)) {
            return [util.u8aToString(util.hexToU8a(value)), 0];
        }
        return [value ? value.toString() : '', 0];
    }
    class Text extends String {
        registry;
        createdAtHash;
        initialU8aLength;
        isStorageFallback;
        #override = null;
        constructor(registry, value) {
            const [str, decodedLength] = decodeText(value);
            super(str);
            this.registry = registry;
            this.initialU8aLength = decodedLength;
        }
        get encodedLength() {
            return this.toU8a().length;
        }
        get hash() {
            return this.registry.hash(this.toU8a());
        }
        get isEmpty() {
            return this.length === 0;
        }
        get length() {
            return super.length;
        }
        eq(other) {
            return util.isString(other)
                ? this.toString() === other.toString()
                : false;
        }
        inspect() {
            const value = util.stringToU8a(super.toString());
            return {
                outer: value.length
                    ? [util.compactToU8a(value.length), value]
                    : [util.compactToU8a(value.length)]
            };
        }
        setOverride(override) {
            this.#override = override;
        }
        toHex() {
            return util.u8aToHex(this.toU8a(true));
        }
        toHuman() {
            return this.toJSON();
        }
        toJSON() {
            return this.toString();
        }
        toPrimitive() {
            return this.toJSON();
        }
        toRawType() {
            return 'Text';
        }
        toString() {
            return this.#override || super.toString();
        }
        toU8a(isBare) {
            const encoded = util.stringToU8a(super.toString());
            return isBare
                ? encoded
                : util.compactAddLength(encoded);
        }
    }

    class Type extends Text {
        constructor(registry, value = '') {
            super(registry, value);
            this.setOverride(sanitize(this.toString()));
        }
        toRawType() {
            return 'Type';
        }
    }

    function decodeU8aFixed(value, bitLength) {
        const u8a = util.u8aToU8a(value);
        const byteLength = bitLength / 8;
        if (!u8a.length) {
            return [new Uint8Array(byteLength), 0];
        }
        if (util.isU8a(value) ? u8a.length < byteLength : u8a.length !== byteLength) {
            throw new Error(`Expected input with ${byteLength} bytes (${bitLength} bits), found ${u8a.length} bytes`);
        }
        return [u8a.subarray(0, byteLength), byteLength];
    }
    class U8aFixed extends Raw {
        constructor(registry, value = new Uint8Array(), bitLength = 256) {
            const [u8a, decodedLength] = decodeU8aFixed(value, bitLength);
            super(registry, u8a, decodedLength);
        }
        static with(bitLength, typeName) {
            return class extends U8aFixed {
                constructor(registry, value) {
                    super(registry, value, bitLength);
                }
                toRawType() {
                    return typeName || super.toRawType();
                }
            };
        }
        toRawType() {
            return `[u8;${this.length}]`;
        }
    }

    function decodeRaw(registry, typeName, value) {
        const Type = typeToConstructor(registry, typeName);
        if (util.isU8a(value) || util.isHex(value)) {
            try {
                const [, u8a] = util.isHex(value)
                    ? [0, util.u8aToU8a(value)]
                    : (value instanceof Raw)
                        ? [0, value.subarray()]
                        : util.compactStripLength(value);
                return [Type, new Type(registry, u8a), value];
            }
            catch {
                return [Type, null, value];
            }
        }
        const instance = new Type(registry, value);
        return [Type, instance, util.compactAddLength(instance.toU8a())];
    }
    class WrapperKeepOpaque extends Bytes {
        #Type;
        #decoded;
        #opaqueName;
        constructor(registry, typeName, value, { opaqueName = 'WrapperKeepOpaque' } = {}) {
            const [Type, decoded, u8a] = decodeRaw(registry, typeName, value);
            super(registry, u8a);
            this.#Type = Type;
            this.#decoded = decoded;
            this.#opaqueName = opaqueName;
        }
        static with(Type) {
            return class extends WrapperKeepOpaque {
                constructor(registry, value) {
                    super(registry, Type, value);
                }
            };
        }
        get isDecoded() {
            return !!this.#decoded;
        }
        inspect() {
            return this.#decoded
                ? {
                    inner: [this.#decoded.inspect()],
                    outer: [util.compactToU8a(this.length)]
                }
                : {
                    outer: [util.compactToU8a(this.length), this.toU8a(true)]
                };
        }
        toHuman(isExtended, disableAscii) {
            return this.#decoded
                ? this.#decoded.toHuman(isExtended, disableAscii)
                : super.toHuman(isExtended, disableAscii);
        }
        toPrimitive(disableAscii) {
            return this.#decoded
                ? this.#decoded.toPrimitive(disableAscii)
                : super.toPrimitive(disableAscii);
        }
        toRawType() {
            return `${this.#opaqueName}<${this.registry.getClassName(this.#Type) || (this.#decoded ? this.#decoded.toRawType() : new this.#Type(this.registry).toRawType())}>`;
        }
        toString() {
            return this.#decoded
                ? this.#decoded.toString()
                : super.toString();
        }
        unwrap() {
            if (!this.#decoded) {
                throw new Error(`${this.#opaqueName}: unwrapping an undecodable value`);
            }
            return this.#decoded;
        }
    }

    class WrapperOpaque extends WrapperKeepOpaque {
        constructor(registry, typeName, value) {
            super(registry, typeName, value, { opaqueName: 'WrapperOpaque' });
        }
        static with(Type) {
            return class extends WrapperOpaque {
                constructor(registry, value) {
                    super(registry, Type, value);
                }
            };
        }
        get inner() {
            return this.unwrap();
        }
    }

    class Float extends Number {
        encodedLength;
        registry;
        createdAtHash;
        initialU8aLength;
        isStorageFallback;
        #bitLength;
        constructor(registry, value, { bitLength = 32 } = {}) {
            super(util.isU8a(value) || util.isHex(value)
                ? value.length === 0
                    ? 0
                    : util.u8aToFloat(util.u8aToU8a(value), { bitLength })
                : (value || 0));
            this.#bitLength = bitLength;
            this.encodedLength = bitLength / 8;
            this.initialU8aLength = this.encodedLength;
            this.registry = registry;
        }
        static with(bitLength) {
            return class extends Float {
                constructor(registry, value) {
                    super(registry, value, { bitLength });
                }
            };
        }
        get hash() {
            return this.registry.hash(this.toU8a());
        }
        get isEmpty() {
            return this.valueOf() === 0;
        }
        eq(other) {
            return this.valueOf() === Number(other);
        }
        inspect() {
            return {
                outer: [this.toU8a()]
            };
        }
        toHex() {
            return util.u8aToHex(this.toU8a());
        }
        toHuman() {
            return this.toString();
        }
        toJSON() {
            return this.toHex();
        }
        toNumber() {
            return this.valueOf();
        }
        toPrimitive() {
            return this.toNumber();
        }
        toRawType() {
            return `f${this.#bitLength}`;
        }
        toU8a(_isBare) {
            return util.floatToU8a(this, {
                bitLength: this.#bitLength
            });
        }
    }

    function decodeJson(value) {
        return Object.entries(value || {});
    }
    class Json extends Map {
        registry;
        createdAtHash;
        initialU8aLength;
        isStorageFallback;
        constructor(registry, value) {
            const decoded = decodeJson(value);
            super(decoded);
            this.registry = registry;
            util.objectProperties(this, decoded.map(([k]) => k), (k) => this.get(k));
        }
        get encodedLength() {
            return 0 | 0;
        }
        get hash() {
            return this.registry.hash(this.toU8a());
        }
        get isEmpty() {
            return [...this.keys()].length === 0;
        }
        eq(other) {
            return compareMap(this, other);
        }
        getT(key) {
            return this.get(key);
        }
        inspect() {
            throw new Error('Unimplemented');
        }
        toHex() {
            throw new Error('Unimplemented');
        }
        toHuman() {
            return [...this.entries()].reduce((json, [key, value]) => {
                json[key] = util.isFunction(value?.toHuman)
                    ? value.toHuman()
                    : value;
                return json;
            }, {});
        }
        toJSON() {
            return [...this.entries()].reduce((json, [key, value]) => {
                json[key] = value;
                return json;
            }, {});
        }
        toPrimitive(disableAscii) {
            return [...this.entries()].reduce((json, [key, value]) => {
                json[key] = util.isFunction(value.toPrimitive)
                    ? value.toPrimitive(disableAscii)
                    : value;
                return json;
            }, {});
        }
        toRawType() {
            return 'Json';
        }
        toString() {
            return util.stringify(this.toJSON());
        }
        toU8a(_isBare) {
            throw new Error('Unimplemented');
        }
    }

    function encodeSet(setValues, values) {
        const encoded = new util.BN(0);
        for (let i = 0, count = values.length; i < count; i++) {
            encoded.ior(util.bnToBn(setValues[values[i]] || 0));
        }
        return encoded;
    }
    function decodeSetArray(setValues, values) {
        const count = values.length;
        const result = new Array(count);
        for (let i = 0; i < count; i++) {
            const key = values[i];
            if (util.isUndefined(setValues[key])) {
                throw new Error(`Set: Invalid key '${key}' passed to Set, allowed ${Object.keys(setValues).join(', ')}`);
            }
            result[i] = key;
        }
        return result;
    }
    function decodeSetNumber(setValues, _value) {
        const bn = util.bnToBn(_value);
        const keys = Object.keys(setValues);
        const result = [];
        for (let i = 0, count = keys.length; i < count; i++) {
            const key = keys[i];
            if (bn.and(util.bnToBn(setValues[key])).eq(util.bnToBn(setValues[key]))) {
                result.push(key);
            }
        }
        const computed = encodeSet(setValues, result);
        if (!bn.eq(computed)) {
            throw new Error(`Set: Mismatch decoding '${bn.toString()}', computed as '${computed.toString()}' with ${result.join(', ')}`);
        }
        return result;
    }
    function decodeSet(setValues, value = 0, bitLength) {
        if (bitLength % 8 !== 0) {
            throw new Error(`Expected valid bitLength, power of 8, found ${bitLength}`);
        }
        const byteLength = bitLength / 8;
        if (util.isU8a(value)) {
            return value.length === 0
                ? []
                : decodeSetNumber(setValues, util.u8aToBn(value.subarray(0, byteLength), { isLe: true }));
        }
        else if (util.isString(value)) {
            return decodeSet(setValues, util.u8aToU8a(value), byteLength);
        }
        else if (value instanceof Set || Array.isArray(value)) {
            const input = Array.isArray(value)
                ? value
                : [...value.values()];
            return decodeSetArray(setValues, input);
        }
        return decodeSetNumber(setValues, value);
    }
    class CodecSet extends Set {
        registry;
        createdAtHash;
        initialU8aLength;
        isStorageFallback;
        #allowed;
        #byteLength;
        constructor(registry, setValues, value, bitLength = 8) {
            super(decodeSet(setValues, value, bitLength));
            this.registry = registry;
            this.#allowed = setValues;
            this.#byteLength = bitLength / 8;
        }
        static with(values, bitLength) {
            return class extends CodecSet {
                static {
                    const keys = Object.keys(values);
                    const count = keys.length;
                    const isKeys = new Array(count);
                    for (let i = 0; i < count; i++) {
                        isKeys[i] = `is${util.stringPascalCase(keys[i])}`;
                    }
                    util.objectProperties(this.prototype, isKeys, (_, i, self) => self.strings.includes(keys[i]));
                }
                constructor(registry, value) {
                    super(registry, values, value, bitLength);
                }
            };
        }
        get encodedLength() {
            return this.#byteLength;
        }
        get hash() {
            return this.registry.hash(this.toU8a());
        }
        get isEmpty() {
            return this.size === 0;
        }
        get strings() {
            return [...super.values()];
        }
        get valueEncoded() {
            return encodeSet(this.#allowed, this.strings);
        }
        add = (key) => {
            if (this.#allowed && util.isUndefined(this.#allowed[key])) {
                throw new Error(`Set: Invalid key '${key}' on add`);
            }
            super.add(key);
            return this;
        };
        eq(other) {
            if (Array.isArray(other)) {
                return compareArray(this.strings.sort(), other.sort());
            }
            else if (other instanceof Set) {
                return this.eq([...other.values()]);
            }
            else if (util.isNumber(other) || util.isBn(other)) {
                return this.valueEncoded.eq(util.bnToBn(other));
            }
            return false;
        }
        inspect() {
            return {
                outer: [this.toU8a()]
            };
        }
        toHex() {
            return util.u8aToHex(this.toU8a());
        }
        toHuman() {
            return this.toJSON();
        }
        toJSON() {
            return this.strings;
        }
        toNumber() {
            return this.valueEncoded.toNumber();
        }
        toPrimitive() {
            return this.toJSON();
        }
        toRawType() {
            return util.stringify({ _set: this.#allowed });
        }
        toString() {
            return `[${this.strings.join(', ')}]`;
        }
        toU8a(_isBare) {
            return util.bnToU8a(this.valueEncoded, {
                bitLength: this.#byteLength * 8,
                isLe: true
            });
        }
    }

    class f32 extends Float.with(32) {
        __FloatType = 'f32';
    }

    class f64 extends Float.with(64) {
        __FloatType = 'f64';
    }

    class i8 extends Int.with(8) {
        __IntType = 'i8';
    }

    class i16 extends Int.with(16) {
        __IntType = 'i16';
    }

    class i32 extends Int.with(32) {
        __IntType = 'i32';
    }

    class i64 extends Int.with(64) {
        __IntType = 'i64';
    }

    class i128 extends Int.with(128) {
        __IntType = 'i128';
    }

    class i256 extends Int.with(256) {
        __IntType = 'i256';
    }

    class isize extends i32 {
        constructor(registry, value) {
            super(registry, value);
            throw new Error('The `isize` type should not be used. Since it is platform-specific, it creates incompatibilities between native (generally i64) and WASM (always i32) code. Use one of the `i32` or `i64` types explicitly.');
        }
    }

    class u8 extends UInt.with(8) {
        __UIntType = 'u8';
    }

    class u16 extends UInt.with(16) {
        __UIntType = 'u16';
    }

    class u32 extends UInt.with(32) {
        __UIntType = 'u32';
    }

    class u64 extends UInt.with(64) {
        __UIntType = 'u64';
    }

    class u128 extends UInt.with(128) {
        __UIntType = 'u128';
    }

    class u256 extends UInt.with(256) {
        __UIntType = 'u256';
    }

    class usize extends u32 {
        constructor(registry, value) {
            super(registry, value);
            throw new Error('The `usize` type should not be used. Since it is platform-specific, it creates incompatibilities between native (generally u64) and WASM (always u32) code. Use one of the `u32` or `u64` types explicitly.');
        }
    }

    function typeSplit(type) {
        const result = [];
        let c = 0;
        let f = 0;
        let s = 0;
        let t = 0;
        let start = 0;
        for (let i = 0, count = type.length; i < count; i++) {
            switch (type[i]) {
                case ',': {
                    if (!(c || f || s || t)) {
                        result.push(type.substring(start, i).trim());
                        start = i + 1;
                    }
                    break;
                }
                case '<':
                    c++;
                    break;
                case '>':
                    c--;
                    break;
                case '[':
                    f++;
                    break;
                case ']':
                    f--;
                    break;
                case '{':
                    s++;
                    break;
                case '}':
                    s--;
                    break;
                case '(':
                    t++;
                    break;
                case ')':
                    t--;
                    break;
            }
        }
        if (c || f || s || t) {
            throw new Error(`Invalid definition (missing terminators) found in ${type}`);
        }
        result.push(type.substring(start, type.length).trim());
        return result;
    }

    const KNOWN_INTERNALS = ['_alias', '_fallback'];
    function getTypeString(typeOrObj) {
        return util.isString(typeOrObj)
            ? typeOrObj.toString()
            : util.stringify(typeOrObj);
    }
    function isRustEnum(details) {
        const values = Object.values(details);
        if (values.some((v) => util.isNumber(v))) {
            if (!values.every((v) => util.isNumber(v) && v >= 0 && v <= 255)) {
                throw new Error('Invalid number-indexed enum definition');
            }
            return false;
        }
        return true;
    }
    function _decodeEnum(value, details, count, fallbackType) {
        value.info = exports.TypeDefInfo.Enum;
        value.fallbackType = fallbackType;
        if (Array.isArray(details)) {
            value.sub = details.map((name, index) => ({
                index,
                info: exports.TypeDefInfo.Plain,
                name,
                type: 'Null'
            }));
        }
        else if (isRustEnum(details)) {
            value.sub = Object.entries(details).map(([name, typeOrObj], index) => util.objectSpread({}, getTypeDef(getTypeString(typeOrObj || 'Null'), { name }, count), { index }));
        }
        else {
            value.sub = Object.entries(details).map(([name, index]) => ({
                index,
                info: exports.TypeDefInfo.Plain,
                name,
                type: 'Null'
            }));
        }
        return value;
    }
    function _decodeSet(value, details, fallbackType) {
        value.info = exports.TypeDefInfo.Set;
        value.fallbackType = fallbackType;
        value.length = details._bitLength;
        value.sub = Object
            .entries(details)
            .filter(([name]) => !name.startsWith('_'))
            .map(([name, index]) => ({
            index,
            info: exports.TypeDefInfo.Plain,
            name,
            type: 'Null'
        }));
        return value;
    }
    function _decodeStruct(value, type, _, count) {
        const parsed = JSON.parse(type);
        const keys = Object.keys(parsed);
        if (parsed._enum) {
            return _decodeEnum(value, parsed._enum, count, parsed._fallback);
        }
        else if (parsed._set) {
            return _decodeSet(value, parsed._set, parsed._fallback);
        }
        value.alias = parsed._alias
            ? new Map(Object.entries(parsed._alias))
            : undefined;
        value.fallbackType = parsed._fallback;
        value.sub = keys
            .filter((name) => !KNOWN_INTERNALS.includes(name))
            .map((name) => getTypeDef(getTypeString(parsed[name]), { name }, count));
        return value;
    }
    function _decodeFixedVec(value, type, _, count) {
        const max = type.length - 1;
        let index = -1;
        let inner = 0;
        for (let i = 1; (i < max) && (index === -1); i++) {
            switch (type[i]) {
                case ';': {
                    if (inner === 0) {
                        index = i;
                    }
                    break;
                }
                case '[':
                case '(':
                case '<':
                    inner++;
                    break;
                case ']':
                case ')':
                case '>':
                    inner--;
                    break;
            }
        }
        if (index === -1) {
            throw new Error(`${type}: Unable to extract location of ';'`);
        }
        const vecType = type.substring(1, index);
        const [strLength, displayName] = type.substring(index + 1, max).split(';');
        const length = parseInt(strLength.trim(), 10);
        if (length > 2048) {
            throw new Error(`${type}: Only support for [Type; <length>], where length <= 2048`);
        }
        value.displayName = displayName;
        value.length = length;
        value.sub = getTypeDef(vecType, {}, count);
        return value;
    }
    function _decodeTuple(value, _, subType, count) {
        value.sub = subType.length === 0
            ? []
            : typeSplit(subType).map((inner) => getTypeDef(inner, {}, count));
        return value;
    }
    function _decodeAnyInt(value, type, _, clazz) {
        const [strLength, displayName] = type.substring(clazz.length + 1, type.length - 1).split(',');
        const length = parseInt(strLength.trim(), 10);
        if ((length > 8192) || (length % 8)) {
            throw new Error(`${type}: Only support for ${clazz}<bitLength>, where length <= 8192 and a power of 8, found ${length}`);
        }
        value.displayName = displayName;
        value.length = length;
        return value;
    }
    function _decodeInt(value, type, subType) {
        return _decodeAnyInt(value, type, subType, 'Int');
    }
    function _decodeUInt(value, type, subType) {
        return _decodeAnyInt(value, type, subType, 'UInt');
    }
    function _decodeDoNotConstruct(value, type, _) {
        const NAME_LENGTH = 'DoNotConstruct'.length;
        value.displayName = type.substring(NAME_LENGTH + 1, type.length - 1);
        return value;
    }
    function hasWrapper(type, [start, end]) {
        return (type.startsWith(start)) && (type.slice(-1 * end.length) === end);
    }
    const nestedExtraction = [
        ['[', ']', exports.TypeDefInfo.VecFixed, _decodeFixedVec],
        ['{', '}', exports.TypeDefInfo.Struct, _decodeStruct],
        ['(', ')', exports.TypeDefInfo.Tuple, _decodeTuple],
        ['BTreeMap<', '>', exports.TypeDefInfo.BTreeMap, _decodeTuple],
        ['HashMap<', '>', exports.TypeDefInfo.HashMap, _decodeTuple],
        ['Int<', '>', exports.TypeDefInfo.Int, _decodeInt],
        ['Result<', '>', exports.TypeDefInfo.Result, _decodeTuple],
        ['UInt<', '>', exports.TypeDefInfo.UInt, _decodeUInt],
        ['DoNotConstruct<', '>', exports.TypeDefInfo.DoNotConstruct, _decodeDoNotConstruct]
    ];
    const wrappedExtraction = [
        ['BTreeSet<', '>', exports.TypeDefInfo.BTreeSet],
        ['Compact<', '>', exports.TypeDefInfo.Compact],
        ['Linkage<', '>', exports.TypeDefInfo.Linkage],
        ['Opaque<', '>', exports.TypeDefInfo.WrapperOpaque],
        ['Option<', '>', exports.TypeDefInfo.Option],
        ['Range<', '>', exports.TypeDefInfo.Range],
        ['RangeInclusive<', '>', exports.TypeDefInfo.RangeInclusive],
        ['Vec<', '>', exports.TypeDefInfo.Vec],
        ['WrapperKeepOpaque<', '>', exports.TypeDefInfo.WrapperKeepOpaque],
        ['WrapperOpaque<', '>', exports.TypeDefInfo.WrapperOpaque]
    ];
    function extractSubType(type, [start, end]) {
        return type.substring(start.length, type.length - end.length);
    }
    function getTypeDef(_type, { displayName, name } = {}, count = 0) {
        const type = sanitize(_type);
        const value = { displayName, info: exports.TypeDefInfo.Plain, name, type };
        if (++count > 64) {
            throw new Error('getTypeDef: Maximum nested limit reached');
        }
        const nested = nestedExtraction.find((nested) => hasWrapper(type, nested));
        if (nested) {
            value.info = nested[2];
            return nested[3](value, type, extractSubType(type, nested), count);
        }
        const wrapped = wrappedExtraction.find((wrapped) => hasWrapper(type, wrapped));
        if (wrapped) {
            value.info = wrapped[2];
            value.sub = getTypeDef(extractSubType(type, wrapped), {}, count);
        }
        return value;
    }

    function getTypeDefType({ lookupName, type }) {
        return lookupName || type;
    }
    function getSubDefArray(value) {
        if (!Array.isArray(value.sub)) {
            throw new Error(`Expected subtype as TypeDef[] in ${util.stringify(value)}`);
        }
        return value.sub;
    }
    function getSubDef(value) {
        if (!value.sub || Array.isArray(value.sub)) {
            throw new Error(`Expected subtype as TypeDef in ${util.stringify(value)}`);
        }
        return value.sub;
    }
    function getSubType(value) {
        return getTypeDefType(getSubDef(value));
    }
    function getTypeClassMap(value) {
        const subs = getSubDefArray(value);
        const map = {};
        for (let i = 0, count = subs.length; i < count; i++) {
            const sub = subs[i];
            if (!sub.name) {
                throw new Error(`No name found in definition ${util.stringify(sub)}`);
            }
            map[sub.name] = getTypeDefType(sub);
        }
        return map;
    }
    function getTypeClassArray(value) {
        return getSubDefArray(value).map(getTypeDefType);
    }
    function createInt(Clazz, { displayName, length }) {
        if (!util.isNumber(length)) {
            throw new Error(`Expected bitLength information for ${displayName || Clazz.constructor.name}<bitLength>`);
        }
        return Clazz.with(length, displayName);
    }
    function createHashMap(Clazz, value) {
        const [keyType, valueType] = getTypeClassArray(value);
        return Clazz.with(keyType, valueType);
    }
    function createWithSub(Clazz, value) {
        return Clazz.with(getSubType(value));
    }
    const infoMapping = {
        [exports.TypeDefInfo.BTreeMap]: (_registry, value) => createHashMap(BTreeMap, value),
        [exports.TypeDefInfo.BTreeSet]: (_registry, value) => createWithSub(BTreeSet, value),
        [exports.TypeDefInfo.Compact]: (_registry, value) => createWithSub(Compact, value),
        [exports.TypeDefInfo.DoNotConstruct]: (_registry, value) => DoNotConstruct.with(value.displayName || value.type),
        [exports.TypeDefInfo.Enum]: (_registry, value) => {
            const subs = getSubDefArray(value);
            return Enum.with(subs.every(({ type }) => type === 'Null')
                ? subs.reduce((out, { index, name }, count) => {
                    if (!name) {
                        throw new Error('No name found in sub definition');
                    }
                    out[name] = index || count;
                    return out;
                }, {})
                : getTypeClassMap(value));
        },
        [exports.TypeDefInfo.HashMap]: (_registry, value) => createHashMap(HashMap, value),
        [exports.TypeDefInfo.Int]: (_registry, value) => createInt(Int, value),
        [exports.TypeDefInfo.Linkage]: (_registry, value) => {
            const type = `Option<${getSubType(value)}>`;
            const Clazz = Struct.with({ previous: type, next: type });
            Clazz.prototype.toRawType = function () {
                return `Linkage<${this.next.toRawType(true)}>`;
            };
            return Clazz;
        },
        [exports.TypeDefInfo.Null]: (_registry, _value) => Null,
        [exports.TypeDefInfo.Option]: (_registry, value) => {
            if (!value.sub || Array.isArray(value.sub)) {
                throw new Error('Expected type information for Option');
            }
            return createWithSub(Option, value);
        },
        [exports.TypeDefInfo.Plain]: (registry, value) => registry.getOrUnknown(value.type),
        [exports.TypeDefInfo.Range]: (_registry, value) => createWithSub(Range, value),
        [exports.TypeDefInfo.RangeInclusive]: (_registry, value) => createWithSub(RangeInclusive, value),
        [exports.TypeDefInfo.Result]: (_registry, value) => {
            const [Ok, Err] = getTypeClassArray(value);
            return Result.with({ Err, Ok });
        },
        [exports.TypeDefInfo.Set]: (_registry, value) => CodecSet.with(getSubDefArray(value).reduce((result, { index, name }) => {
            if (!name || !util.isNumber(index)) {
                throw new Error('No name found in sub definition');
            }
            result[name] = index;
            return result;
        }, {}), value.length),
        [exports.TypeDefInfo.Si]: (registry, value) => getTypeClass(registry, registry.lookup.getTypeDef(value.type)),
        [exports.TypeDefInfo.Struct]: (_registry, value) => Struct.with(getTypeClassMap(value), value.alias),
        [exports.TypeDefInfo.Tuple]: (_registry, value) => Tuple.with(getTypeClassArray(value)),
        [exports.TypeDefInfo.UInt]: (_registry, value) => createInt(UInt, value),
        [exports.TypeDefInfo.Vec]: (_registry, { sub }) => {
            if (!sub || Array.isArray(sub)) {
                throw new Error('Expected type information for vector');
            }
            return (sub.type === 'u8'
                ? Bytes
                : Vec.with(getTypeDefType(sub)));
        },
        [exports.TypeDefInfo.VecFixed]: (_registry, { displayName, length, sub }) => {
            if (!util.isNumber(length) || !sub || Array.isArray(sub)) {
                throw new Error('Expected length & type information for fixed vector');
            }
            return (sub.type === 'u8'
                ? U8aFixed.with((length * 8), displayName)
                : VecFixed.with(getTypeDefType(sub), length));
        },
        [exports.TypeDefInfo.WrapperKeepOpaque]: (_registry, value) => createWithSub(WrapperKeepOpaque, value),
        [exports.TypeDefInfo.WrapperOpaque]: (_registry, value) => createWithSub(WrapperOpaque, value)
    };
    function constructTypeClass(registry, typeDef) {
        try {
            const Type = infoMapping[typeDef.info](registry, typeDef);
            if (!Type) {
                throw new Error('No class created');
            }
            if (!Type.__fallbackType && typeDef.fallbackType) {
                Type.__fallbackType = typeDef.fallbackType;
            }
            return Type;
        }
        catch (error) {
            throw new Error(`Unable to construct class from ${util.stringify(typeDef)}: ${error.message}`);
        }
    }
    function getTypeClass(registry, typeDef) {
        return registry.getUnsafe(typeDef.type, false, typeDef);
    }
    function createClassUnsafe(registry, type) {
        return (
        registry.getUnsafe(type) ||
            getTypeClass(registry, registry.isLookupType(type)
                ? registry.lookup.getTypeDef(type)
                : getTypeDef(type)));
    }

    function checkInstance(created, matcher) {
        const u8a = created.toU8a();
        const rawType = created.toRawType();
        const isOk = (
        util.u8aEq(u8a, matcher) ||
            (
            ['Bytes', 'Text', 'Type'].includes(rawType) &&
                matcher.length === created.length) ||
            (
            created.isEmpty &&
                matcher.every((v) => !v)));
        if (!isOk) {
            throw new Error(`${rawType}:: Decoded input doesn't match input, received ${util.u8aToHex(matcher, 512)} (${matcher.length} bytes), created ${util.u8aToHex(u8a, 512)} (${u8a.length} bytes)`);
        }
    }
    function checkPedantic(created, [value]) {
        if (util.isU8a(value)) {
            checkInstance(created, value);
        }
        else if (util.isHex(value)) {
            checkInstance(created, util.u8aToU8a(value));
        }
    }
    function initType(registry, Type, params = [], { blockHash, isFallback, isOptional, isPedantic } = {}) {
        const created = new (isOptional
            ? Option.with(Type)
            : Type)(registry, ...params);
        isPedantic && checkPedantic(created, params);
        if (blockHash) {
            created.createdAtHash = createTypeUnsafe(registry, 'BlockHash', [blockHash]);
        }
        if (isFallback) {
            created.isStorageFallback = true;
        }
        return created;
    }
    function createTypeUnsafe(registry, type, params = [], options = {}) {
        let Clazz = null;
        let firstError = null;
        try {
            Clazz = createClassUnsafe(registry, type);
            return initType(registry, Clazz, params, options);
        }
        catch (error) {
            firstError = new Error(`createType(${type}):: ${error.message}`);
        }
        if (Clazz?.__fallbackType) {
            try {
                Clazz = createClassUnsafe(registry, Clazz.__fallbackType);
                return initType(registry, Clazz, params, options);
            }
            catch {
            }
        }
        throw firstError;
    }

    const stringIdentity = (value) => value.toString();
    const INFO_WRAP = ['BTreeMap', 'BTreeSet', 'Compact', 'HashMap', 'Option', 'Result', 'Vec'];
    function paramsNotation(outer, inner, transform = stringIdentity) {
        return `${outer}${inner
        ? `<${(Array.isArray(inner) ? inner : [inner]).map(transform).join(', ')}>`
        : ''}`;
    }
    function encodeWithParams(registry, typeDef, outer) {
        const { info, sub } = typeDef;
        switch (info) {
            case exports.TypeDefInfo.BTreeMap:
            case exports.TypeDefInfo.BTreeSet:
            case exports.TypeDefInfo.Compact:
            case exports.TypeDefInfo.HashMap:
            case exports.TypeDefInfo.Linkage:
            case exports.TypeDefInfo.Option:
            case exports.TypeDefInfo.Range:
            case exports.TypeDefInfo.RangeInclusive:
            case exports.TypeDefInfo.Result:
            case exports.TypeDefInfo.Vec:
            case exports.TypeDefInfo.WrapperKeepOpaque:
            case exports.TypeDefInfo.WrapperOpaque:
                return paramsNotation(outer, sub, (p) => encodeTypeDef(registry, p));
        }
        throw new Error(`Unable to encode ${util.stringify(typeDef)} with params`);
    }
    function encodeSubTypes(registry, sub, asEnum, extra) {
        const names = sub.map(({ name }) => name);
        if (!names.every((n) => !!n)) {
            throw new Error(`Subtypes does not have consistent names, ${names.join(', ')}`);
        }
        const inner = util.objectSpread({}, extra);
        for (let i = 0, count = sub.length; i < count; i++) {
            const def = sub[i];
            if (!def.name) {
                throw new Error(`No name found in ${util.stringify(def)}`);
            }
            inner[def.name] = encodeTypeDef(registry, def);
        }
        return util.stringify(asEnum
            ? { _enum: inner }
            : inner);
    }
    const encoders = {
        [exports.TypeDefInfo.BTreeMap]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'BTreeMap'),
        [exports.TypeDefInfo.BTreeSet]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'BTreeSet'),
        [exports.TypeDefInfo.Compact]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'Compact'),
        [exports.TypeDefInfo.DoNotConstruct]: (registry, { displayName, lookupIndex, lookupName }) => `DoNotConstruct<${lookupName || displayName || (util.isUndefined(lookupIndex) ? 'Unknown' : registry.createLookupType(lookupIndex))}>`,
        [exports.TypeDefInfo.Enum]: (registry, { sub }) => {
            if (!Array.isArray(sub)) {
                throw new Error('Unable to encode Enum type');
            }
            return sub.every(({ type }) => type === 'Null')
                ? util.stringify({ _enum: sub.map(({ name }, index) => `${name || `Empty${index}`}`) })
                : encodeSubTypes(registry, sub, true);
        },
        [exports.TypeDefInfo.HashMap]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'HashMap'),
        [exports.TypeDefInfo.Int]: (_registry, { length = 32 }) => `Int<${length}>`,
        [exports.TypeDefInfo.Linkage]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'Linkage'),
        [exports.TypeDefInfo.Null]: (_registry, _typeDef) => 'Null',
        [exports.TypeDefInfo.Option]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'Option'),
        [exports.TypeDefInfo.Plain]: (_registry, { displayName, type }) => displayName || type,
        [exports.TypeDefInfo.Range]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'Range'),
        [exports.TypeDefInfo.RangeInclusive]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'RangeInclusive'),
        [exports.TypeDefInfo.Result]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'Result'),
        [exports.TypeDefInfo.Set]: (_registry, { length = 8, sub }) => {
            if (!Array.isArray(sub)) {
                throw new Error('Unable to encode Set type');
            }
            return util.stringify({
                _set: sub.reduce((all, { index, name }, count) => util.objectSpread(all, { [`${name || `Unknown${index || count}`}`]: index || count }), { _bitLength: length || 8 })
            });
        },
        [exports.TypeDefInfo.Si]: (_registry, { lookupName, type }) => lookupName || type,
        [exports.TypeDefInfo.Struct]: (registry, { alias, sub }) => {
            if (!Array.isArray(sub)) {
                throw new Error('Unable to encode Struct type');
            }
            return encodeSubTypes(registry, sub, false, alias
                ? {
                    _alias: [...alias.entries()].reduce((all, [k, v]) => util.objectSpread(all, { [k]: v }), {})
                }
                : {});
        },
        [exports.TypeDefInfo.Tuple]: (registry, { sub }) => {
            if (!Array.isArray(sub)) {
                throw new Error('Unable to encode Tuple type');
            }
            return `(${sub.map((type) => encodeTypeDef(registry, type)).join(',')})`;
        },
        [exports.TypeDefInfo.UInt]: (_registry, { length = 32 }) => `UInt<${length}>`,
        [exports.TypeDefInfo.Vec]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'Vec'),
        [exports.TypeDefInfo.VecFixed]: (_registry, { length, sub }) => {
            if (!util.isNumber(length) || !sub || Array.isArray(sub)) {
                throw new Error('Unable to encode VecFixed type');
            }
            return `[${sub.type};${length}]`;
        },
        [exports.TypeDefInfo.WrapperKeepOpaque]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'WrapperKeepOpaque'),
        [exports.TypeDefInfo.WrapperOpaque]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'WrapperOpaque')
    };
    function encodeType(registry, typeDef, withLookup = true) {
        return withLookup && typeDef.lookupName
            ? typeDef.lookupName
            : encoders[typeDef.info](registry, typeDef);
    }
    function encodeTypeDef(registry, typeDef) {
        return (typeDef.displayName && !INFO_WRAP.some((i) => typeDef.displayName === i))
            ? typeDef.displayName
            : encodeType(registry, typeDef);
    }
    function withTypeString(registry, typeDef) {
        return util.objectSpread({}, typeDef, {
            type: encodeType(registry, typeDef, false)
        });
    }

    const XCM_MAPPINGS = ['AssetInstance', 'Fungibility', 'Junction', 'Junctions', 'MultiAsset', 'MultiAssetFilter', 'MultiLocation', 'Response', 'WildFungibility', 'WildMultiAsset', 'Xcm', 'XcmError'];
    function mapXcmTypes(version) {
        return XCM_MAPPINGS.reduce((all, key) => util.objectSpread(all, { [key]: `${key}${version}` }), {});
    }

    const v0 = {
        FungibilityV0: 'FungibilityV1',
        WildFungibilityV0: 'WildFungibilityV1',
        AssetInstanceV0: {
            _enum: {
                Undefined: 'Null',
                Index8: 'u8',
                Index16: 'Compact<u16>',
                Index32: 'Compact<u32>',
                Index64: 'Compact<u64>',
                Index128: 'Compact<u128>',
                Array4: '[u8; 4]',
                Array8: '[u8; 8]',
                Array16: '[u8; 16]',
                Array32: '[u8; 32]',
                Blob: 'Vec<u8>'
            }
        },
        JunctionV0: {
            _enum: {
                Parent: 'Null',
                Parachain: 'Compact<u32>',
                AccountId32: {
                    network: 'NetworkId',
                    id: 'AccountId'
                },
                AccountIndex64: {
                    network: 'NetworkId',
                    index: 'Compact<u64>'
                },
                AccountKey20: {
                    network: 'NetworkId',
                    key: '[u8; 20]'
                },
                PalletInstance: 'u8',
                GeneralIndex: 'Compact<u128>',
                GeneralKey: 'Vec<u8>',
                OnlyChild: 'Null',
                Plurality: {
                    id: 'BodyId',
                    part: 'BodyPart'
                }
            }
        },
        MultiAssetV0: {
            _enum: {
                None: 'Null',
                All: 'Null',
                AllFungible: 'Null',
                AllNonFungible: 'Null',
                AllAbstractFungible: 'Vec<u8>',
                AllAbstractNonFungible: 'Vec<u8>',
                AllConcreteFungible: 'MultiLocationV0',
                AllConcreteNonFungible: 'MultiLocationV0',
                AbstractFungible: {
                    id: 'Vec<u8>',
                    instance: 'Compact<u128>'
                },
                AbstractNonFungible: {
                    class: 'Vec<u8>',
                    instance: 'AssetInstanceV0'
                },
                ConcreteFungible: {
                    id: 'MultiLocationV0',
                    amount: 'Compact<u128>'
                },
                ConcreteNonFungible: {
                    class: 'MultiLocationV0',
                    instance: 'AssetInstanceV0'
                }
            }
        },
        MultiLocationV0: {
            _enum: {
                Here: 'Null',
                X1: 'JunctionV0',
                X2: '(JunctionV0, JunctionV0)',
                X3: '(JunctionV0, JunctionV0, JunctionV0)',
                X4: '(JunctionV0, JunctionV0, JunctionV0, JunctionV0)',
                X5: '(JunctionV0, JunctionV0, JunctionV0, JunctionV0, JunctionV0)',
                X6: '(JunctionV0, JunctionV0, JunctionV0, JunctionV0, JunctionV0, JunctionV0)',
                X7: '(JunctionV0, JunctionV0, JunctionV0, JunctionV0, JunctionV0, JunctionV0, JunctionV0)',
                X8: '(JunctionV0, JunctionV0, JunctionV0, JunctionV0, JunctionV0, JunctionV0, JunctionV0, JunctionV0)'
            }
        },
        OriginKindV0: {
            _enum: ['Native', 'SovereignAccount', 'Superuser', 'Xcm']
        },
        ResponseV0: {
            _enum: {
                Assets: 'Vec<MultiAssetV0>'
            }
        },
        XcmV0: {
            _enum: {
                WithdrawAsset: {
                    assets: 'Vec<MultiAssetV0>',
                    effects: 'Vec<XcmOrderV0>'
                },
                ReserveAssetDeposit: {
                    assets: 'Vec<MultiAssetV0>',
                    effects: 'Vec<XcmOrderV0>'
                },
                ReceiveTeleportedAsset: {
                    assets: 'Vec<MultiAssetV0>',
                    effects: 'Vec<XcmOrderV0>'
                },
                QueryResponse: {
                    queryId: 'Compact<u64>',
                    response: 'ResponseV0'
                },
                TransferAsset: {
                    assets: 'Vec<MultiAssetV0>',
                    dest: 'MultiLocationV0'
                },
                TransferReserveAsset: {
                    assets: 'Vec<MultiAssetV0>',
                    dest: 'MultiLocationV0',
                    effects: 'Vec<XcmOrderV0>'
                },
                Transact: {
                    originType: 'XcmOriginKind',
                    requireWeightAtMost: 'u64',
                    call: 'DoubleEncodedCall'
                },
                HrmpNewChannelOpenRequest: {
                    sender: 'Compact<u32>',
                    maxMessageSize: 'Compact<u32>',
                    maxCapacity: 'Compact<u32>'
                },
                HrmpChannelAccepted: {
                    recipient: 'Compact<u32>'
                },
                HrmpChannelClosing: {
                    initiator: 'Compact<u32>',
                    sender: 'Compact<u32>',
                    recipient: 'Compact<u32>'
                },
                RelayedFrom: {
                    who: 'MultiLocationV0',
                    message: 'XcmV0'
                }
            }
        },
        XcmErrorV0: {
            _enum: {
                Undefined: 'Null',
                Overflow: 'Null',
                Unimplemented: 'Null',
                UnhandledXcmVersion: 'Null',
                UnhandledXcmMessage: 'Null',
                UnhandledEffect: 'Null',
                EscalationOfPrivilege: 'Null',
                UntrustedReserveLocation: 'Null',
                UntrustedTeleportLocation: 'Null',
                DestinationBufferOverflow: 'Null',
                SendFailed: 'Null',
                CannotReachDestination: '(MultiLocation, Xcm)',
                MultiLocationFull: 'Null',
                FailedToDecode: 'Null',
                BadOrigin: 'Null',
                ExceedsMaxMessageSize: 'Null',
                FailedToTransactAsset: 'Null',
                WeightLimitReached: 'Weight',
                Wildcard: 'Null',
                TooMuchWeightRequired: 'Null',
                NotHoldingFees: 'Null',
                WeightNotComputable: 'Null',
                Barrier: 'Null',
                NotWithdrawable: 'Null',
                LocationCannotHold: 'Null',
                TooExpensive: 'Null',
                AssetNotFound: 'Null',
                RecursionLimitReached: 'Null'
            }
        },
        XcmOrderV0: {
            _enum: {
                Null: 'Null',
                DepositAsset: {
                    assets: 'Vec<MultiAssetV0>',
                    dest: 'MultiLocationV0'
                },
                DepositReserveAsset: {
                    assets: 'Vec<MultiAssetV0>',
                    dest: 'MultiLocationV0',
                    effects: 'Vec<XcmOrderV0>'
                },
                ExchangeAsset: {
                    give: 'Vec<MultiAssetV0>',
                    receive: 'Vec<MultiAssetV0>'
                },
                InitiateReserveWithdraw: {
                    assets: 'Vec<MultiAssetV0>',
                    reserve: 'MultiLocationV0',
                    effects: 'Vec<XcmOrderV0>'
                },
                InitiateTeleport: {
                    assets: 'Vec<MultiAssetV0>',
                    dest: 'MultiLocationV0',
                    effects: 'Vec<XcmOrderV0>'
                },
                QueryHolding: {
                    queryId: 'Compact<u64>',
                    dest: 'MultiLocationV0',
                    assets: 'Vec<MultiAssetV0>'
                },
                BuyExecution: {
                    fees: 'MultiAssetV0',
                    weight: 'u64',
                    debt: 'u64',
                    haltOnError: 'bool',
                    xcm: 'Vec<XcmV0>'
                }
            }
        }
    };

    const v1 = {
        AssetInstanceV1: {
            _enum: {
                Undefined: 'Null',
                Index: 'Compact<u128>',
                Array4: '[u8; 4]',
                Array8: '[u8; 8]',
                Array16: '[u8; 16]',
                Array32: '[u8; 32]',
                Blob: 'Bytes'
            }
        },
        FungibilityV1: {
            _enum: {
                Fungible: 'Compact<u128>',
                NonFungible: 'AssetInstanceV1'
            }
        },
        JunctionV1: {
            _enum: {
                Parachain: 'Compact<u32>',
                AccountId32: {
                    network: 'NetworkId',
                    id: 'AccountId'
                },
                AccountIndex64: {
                    network: 'NetworkId',
                    index: 'Compact<u64>'
                },
                AccountKey20: {
                    network: 'NetworkId',
                    key: '[u8; 20]'
                },
                PalletInstance: 'u8',
                GeneralIndex: 'Compact<u128>',
                GeneralKey: 'Vec<u8>',
                OnlyChild: 'Null',
                Plurality: {
                    id: 'BodyId',
                    part: 'BodyPart'
                }
            }
        },
        JunctionsV1: {
            _enum: {
                Here: 'Null',
                X1: 'JunctionV1',
                X2: '(JunctionV1, JunctionV1)',
                X3: '(JunctionV1, JunctionV1, JunctionV1)',
                X4: '(JunctionV1, JunctionV1, JunctionV1, JunctionV1)',
                X5: '(JunctionV1, JunctionV1, JunctionV1, JunctionV1, JunctionV1)',
                X6: '(JunctionV1, JunctionV1, JunctionV1, JunctionV1, JunctionV1, JunctionV1)',
                X7: '(JunctionV1, JunctionV1, JunctionV1, JunctionV1, JunctionV1, JunctionV1, JunctionV1)',
                X8: '(JunctionV1, JunctionV1, JunctionV1, JunctionV1, JunctionV1, JunctionV1, JunctionV1, JunctionV1)'
            }
        },
        MultiAssetsV1: 'Vec<MultiAssetV1>',
        MultiAssetV1: {
            id: 'XcmAssetId',
            fungibility: 'FungibilityV1'
        },
        MultiAssetFilterV1: {
            _enum: {
                Definite: 'MultiAssetsV1',
                Wild: 'WildMultiAssetV1'
            }
        },
        MultiLocationV1: {
            parents: 'u8',
            interior: 'JunctionsV1'
        },
        OriginKindV1: 'OriginKindV0',
        ResponseV1: {
            _enum: {
                Assets: 'MultiAssetsV1'
            }
        },
        WildFungibilityV1: {
            _enum: ['Fungible', 'NonFungible']
        },
        WildMultiAssetV1: {
            _enum: {
                All: 'Null',
                AllOf: {
                    id: 'XcmAssetId',
                    fungibility: 'WildFungibilityV1'
                }
            }
        },
        XcmV1: {
            _enum: {
                WithdrawAsset: {
                    assets: 'MultiAssetsV1',
                    effects: 'Vec<XcmOrderV1>'
                },
                ReserveAssetDeposit: {
                    assets: 'MultiAssetsV1',
                    effects: 'Vec<XcmOrderV1>'
                },
                ReceiveTeleportedAsset: {
                    assets: 'MultiAssetsV1',
                    effects: 'Vec<XcmOrderV1>'
                },
                QueryResponse: {
                    queryId: 'Compact<u64>',
                    response: 'ResponseV1'
                },
                TransferAsset: {
                    assets: 'MultiAssetsV1',
                    dest: 'MultiLocationV1'
                },
                TransferReserveAsset: {
                    assets: 'MultiAssetsV1',
                    dest: 'MultiLocationV1',
                    effects: 'Vec<XcmOrderV1>'
                },
                Transact: {
                    originType: 'XcmOriginKind',
                    requireWeightAtMost: 'u64',
                    call: 'DoubleEncodedCall'
                },
                HrmpNewChannelOpenRequest: {
                    sender: 'Compact<u32>',
                    maxMessageSize: 'Compact<u32>',
                    maxCapacity: 'Compact<u32>'
                },
                HrmpChannelAccepted: {
                    recipient: 'Compact<u32>'
                },
                HrmpChannelClosing: {
                    initiator: 'Compact<u32>',
                    sender: 'Compact<u32>',
                    recipient: 'Compact<u32>'
                },
                RelayedFrom: {
                    who: 'MultiLocationV1',
                    message: 'XcmV1'
                }
            }
        },
        XcmErrorV1: {
            _enum: {
                Undefined: 'Null',
                Overflow: 'Null',
                Unimplemented: 'Null',
                UnhandledXcmVersion: 'Null',
                UnhandledXcmMessage: 'Null',
                UnhandledEffect: 'Null',
                EscalationOfPrivilege: 'Null',
                UntrustedReserveLocation: 'Null',
                UntrustedTeleportLocation: 'Null',
                DestinationBufferOverflow: 'Null',
                SendFailed: 'Null',
                CannotReachDestination: '(MultiLocationV1, XcmV1)',
                MultiLocationFull: 'Null',
                FailedToDecode: 'Null',
                BadOrigin: 'Null',
                ExceedsMaxMessageSize: 'Null',
                FailedToTransactAsset: 'Null',
                WeightLimitReached: 'Weight',
                Wildcard: 'Null',
                TooMuchWeightRequired: 'Null',
                NotHoldingFees: 'Null',
                WeightNotComputable: 'Null',
                Barrier: 'Null',
                NotWithdrawable: 'Null',
                LocationCannotHold: 'Null',
                TooExpensive: 'Null',
                AssetNotFound: 'Null',
                DestinationUnsupported: 'Null',
                RecursionLimitReached: 'Null'
            }
        },
        XcmOrderV1: {
            _enum: {
                Noop: 'Null',
                DepositAsset: {
                    assets: 'MultiAssetFilterV1',
                    maxAssets: 'u32',
                    beneficiary: 'MultiLocationV1'
                },
                DepositReserveAsset: {
                    assets: 'MultiAssetFilterV1',
                    maxAssets: 'u32',
                    dest: 'MultiLocationV1',
                    effects: 'Vec<XcmOrderV1>'
                },
                ExchangeAsset: {
                    give: 'MultiAssetFilterV1',
                    receive: 'MultiAssetsV1'
                },
                InitiateReserveWithdraw: {
                    assets: 'MultiAssetFilterV1',
                    reserve: 'MultiLocationV1',
                    effects: 'Vec<XcmOrderV1>'
                },
                InitiateTeleport: {
                    assets: 'MultiAssetFilterV1',
                    dest: 'MultiLocationV1',
                    effects: 'Vec<XcmOrderV1>'
                },
                QueryHolding: {
                    queryId: 'Compact<u64>',
                    dest: 'MultiLocationV1',
                    assets: 'MultiAssetFilterV1'
                },
                BuyExecution: {
                    fees: 'MultiAssetV1',
                    weight: 'u64',
                    debt: 'u64',
                    haltOnError: 'bool',
                    instructions: 'Vec<XcmV1>'
                }
            }
        }
    };

    const v2 = {
        AssetIdV2: {
            _enum: {
                Concrete: 'MultiLocationV2',
                Abstract: 'Bytes'
            }
        },
        AssetInstanceV2: {
            _enum: {
                Undefined: 'Null',
                Index: 'Compact<u128>',
                Array4: '[u8;4]',
                Array8: '[u8;8]',
                Array16: '[u8;16]',
                Array32: '[u8;32]',
                Blob: 'Bytes'
            }
        },
        BodyIdV2: {
            _enum: {
                Unit: 'Null',
                Named: 'Bytes',
                Index: 'Compact<u32>',
                Executive: 'Null',
                Technical: 'Null',
                Legislative: 'Null',
                Judicial: 'Null',
                Defense: 'Null',
                Administration: 'Null',
                Treasury: 'Null'
            }
        },
        BodyPartV2: {
            _enum: {
                Voice: 'Null',
                Members: {
                    count: 'Compact<u32>'
                },
                Fraction: {
                    nom: 'Compact<u32>',
                    denom: 'Compact<u32>'
                },
                AtLeastProportion: {
                    nom: 'Compact<u32>',
                    denom: 'Compact<u32>'
                },
                MoreThanProportion: {
                    nom: 'Compact<u32>',
                    denom: 'Compact<u32>'
                }
            }
        },
        NetworkIdV2: {
            _enum: {
                Any: 'Null',
                Named: 'Bytes',
                Polkadot: 'Null',
                Kusama: 'Null'
            }
        },
        JunctionV2: {
            _enum: {
                Parachain: 'Compact<u32>',
                AccountId32: {
                    network: 'NetworkIdV2',
                    id: '[u8; 32]'
                },
                AccountIndex64: {
                    network: 'NetworkIdV2',
                    index: 'Compact<u64>'
                },
                AccountKey20: {
                    network: 'NetworkIdV2',
                    key: '[u8; 20]'
                },
                PalletInstance: 'u8',
                GeneralIndex: 'Compact<u128>',
                GeneralKey: 'Bytes',
                OnlyChild: 'Null',
                Plurality: {
                    id: 'BodyIdV2',
                    part: 'BodyPartV2'
                }
            }
        },
        JunctionsV2: {
            _enum: {
                Here: 'Null',
                X1: 'JunctionV2',
                X2: '(JunctionV2, JunctionV2)',
                X3: '(JunctionV2, JunctionV2, JunctionV2)',
                X4: '(JunctionV2, JunctionV2, JunctionV2, JunctionV2)',
                X5: '(JunctionV2, JunctionV2, JunctionV2, JunctionV2, JunctionV2)',
                X6: '(JunctionV2, JunctionV2, JunctionV2, JunctionV2, JunctionV2, JunctionV2)',
                X7: '(JunctionV2, JunctionV2, JunctionV2, JunctionV2, JunctionV2, JunctionV2, JunctionV2)',
                X8: '(JunctionV2, JunctionV2, JunctionV2, JunctionV2, JunctionV2, JunctionV2, JunctionV2, JunctionV2)'
            }
        },
        FungibilityV2: {
            _enum: {
                Fungible: 'Compact<u128>',
                NonFungible: 'AssetInstanceV2'
            }
        },
        InteriorMultiLocationV2: 'JunctionsV2',
        MultiAssetV2: {
            id: 'AssetIdV2',
            fun: 'FungibilityV2'
        },
        MultiAssetsV2: 'Vec<MultiAssetV2>',
        MultiAssetFilterV2: {
            _enum: {
                Definite: 'MultiAssetsV2',
                Wild: 'WildMultiAssetV2'
            }
        },
        MultiLocationV2: {
            parents: 'u8',
            interior: 'JunctionsV2'
        },
        OriginKindV2: {
            _enum: {
                Native: 'Null',
                SovereignAccount: 'Null',
                Superuser: 'Null',
                Xcm: 'Null'
            }
        },
        WildFungibilityV2: {
            _enum: {
                Fungible: 'Null',
                NonFungible: 'Null'
            }
        },
        ResponseV2: {
            _enum: {
                Null: 'Null',
                Assets: 'MultiAssetsV2',
                ExecutionResult: 'Option<(u32, XcmErrorV2)>',
                Version: 'u32'
            }
        },
        ResponseV2Error: '(u32, XcmErrorV2)',
        WeightLimitV2: {
            _enum: {
                Unlimited: 'Null',
                Limited: 'Compact<u64>'
            }
        },
        InstructionV2: {
            _enum: {
                WithdrawAsset: 'MultiAssetsV2',
                ReserveAssetDeposited: 'MultiAssetsV2',
                ReceiveTeleportedAsset: 'MultiAssetsV2',
                QueryResponse: {
                    queryId: 'Compact<u64>',
                    response: 'ResponseV2',
                    maxWeight: 'Compact<u64>'
                },
                TransferAsset: {
                    assets: 'MultiAssetsV2',
                    beneficiary: 'MultiLocationV2'
                },
                TransferReserveAsset: {
                    assets: 'MultiAssetsV2',
                    dest: 'MultiLocationV2',
                    xcm: 'XcmV2'
                },
                Transact: {
                    originType: 'XcmOriginKind',
                    requireWeightAtMost: 'Compact<u64>',
                    call: 'DoubleEncodedCall'
                },
                HrmpNewChannelOpenRequest: {
                    sender: 'Compact<u32>',
                    maxMessageSize: 'Compact<u32>',
                    maxCapacity: 'Compact<u32>'
                },
                HrmpChannelAccepted: {
                    recipient: 'Compact<u32>'
                },
                HrmpChannelClosing: {
                    initiator: 'Compact<u32>',
                    sender: 'Compact<u32>',
                    recipient: 'Compact<u32>'
                },
                ClearOrigin: 'Null',
                DescendOrigin: 'InteriorMultiLocationV2',
                ReportError: {
                    queryId: 'Compact<u64>',
                    dest: 'MultiLocationV2',
                    maxResponseWeight: 'Compact<u64>'
                },
                DepositAsset: {
                    assets: 'MultiAssetFilterV2',
                    maxAssets: 'Compact<u32>',
                    beneficiary: 'MultiLocationV2'
                },
                DepositReserveAsset: {
                    assets: 'MultiAssetFilterV2',
                    maxAssets: 'Compact<u32>',
                    dest: 'MultiLocationV2',
                    xcm: 'XcmV2'
                },
                ExchangeAsset: {
                    give: 'MultiAssetFilterV2',
                    receive: 'MultiAssetsV2'
                },
                InitiateReserveWithdraw: {
                    assets: 'MultiAssetFilterV2',
                    reserve: 'MultiLocationV2',
                    xcm: 'XcmV2'
                },
                InitiateTeleport: {
                    assets: 'MultiAssetFilterV2',
                    dest: 'MultiLocationV2',
                    xcm: 'XcmV2'
                },
                QueryHolding: {
                    queryId: 'Compact<u64>',
                    dest: 'MultiLocationV2',
                    assets: 'MultiAssetFilterV2',
                    maxResponseWeight: 'Compact<u64>'
                },
                BuyExecution: {
                    fees: 'MultiAssetV2',
                    weightLimit: 'WeightLimitV2'
                },
                RefundSurplus: 'Null',
                SetErrorHandler: 'XcmV2<RuntimeCall>',
                SetAppendix: 'XcmV2<RuntimeCall>',
                ClearError: 'Null',
                ClaimAsset: {
                    assets: 'MultiAssetsV2',
                    ticket: 'MultiLocationV2'
                },
                Trap: 'Compact<u64>',
                SubscribeVersion: {
                    queryId: 'Compact<u64>',
                    maxResponseWeight: 'Compact<u64>'
                },
                UnsubscribeVersion: 'Null'
            }
        },
        WildMultiAssetV2: {
            _enum: {
                All: 'Null',
                AllOf: {
                    id: 'AssetIdV2',
                    fun: 'WildFungibilityV2'
                }
            }
        },
        XcmV2: 'Vec<InstructionV2>',
        XcmErrorV2: {
            _enum: {
                Overflow: 'Null',
                Unimplemented: 'Null',
                UntrustedReserveLocation: 'Null',
                UntrustedTeleportLocation: 'Null',
                MultiLocationFull: 'Null',
                MultiLocationNotInvertible: 'Null',
                BadOrigin: 'Null',
                InvalidLocation: 'Null',
                AssetNotFound: 'Null',
                FailedToTransactAsset: 'Null',
                NotWithdrawable: 'Null',
                LocationCannotHold: 'Null',
                ExceedsMaxMessageSize: 'Null',
                DestinationUnsupported: 'Null',
                Transport: 'Null',
                Unroutable: 'Null',
                UnknownClaim: 'Null',
                FailedToDecode: 'Null',
                MaxWeightInvalid: 'Null',
                NotHoldingFees: 'Null',
                TooExpensive: 'Null',
                Trap: 'u64',
                UnhandledXcmVersion: 'Null',
                WeightLimitReached: 'Weight',
                Barrier: 'Null',
                WeightNotComputable: 'Null'
            }
        }
    };

    const v3 = {
        AssetIdV3: {
            _enum: {
                Concrete: 'MultiLocationV3',
                Abstract: 'Bytes'
            }
        },
        BodyIdV3: {
            _enum: {
                Unit: 'Null',
                Moniker: '[u8;4]',
                Index: 'Compact<u32>',
                Executive: 'Null',
                Technical: 'Null',
                Legislative: 'Null',
                Judicial: 'Null',
                Defense: 'Null',
                Administration: 'Null',
                Treasury: 'Null'
            }
        },
        BodyPartV3: {
            _enum: {
                Voice: 'Null',
                Members: {
                    count: 'Compact<u32>'
                },
                Fraction: {
                    nom: 'Compact<u32>',
                    denom: 'Compact<u32>'
                },
                AtLeastProportion: {
                    nom: 'Compact<u32>',
                    denom: 'Compact<u32>'
                },
                MoreThanProportion: {
                    nom: 'Compact<u32>',
                    denom: 'Compact<u32>'
                }
            }
        },
        AssetInstanceV3: {
            _enum: {
                Undefined: 'Null',
                Index: 'Compact<u128>',
                Array4: '[u8;4]',
                Array8: '[u8;8]',
                Array16: '[u8;16]',
                Array32: '[u8;32]'
            }
        },
        FungibilityV3: {
            _enum: {
                Fungible: 'Compact<u128>',
                NonFungible: 'AssetInstanceV3'
            }
        },
        InteriorMultiLocationV3: 'JunctionsV3',
        JunctionV3: {
            _enum: {
                Parachain: 'Compact<u32>',
                AccountId32: {
                    network: 'Option<NetworkIdV3>',
                    id: '[u8;32]'
                },
                AccountIndex64: {
                    network: 'Option<NetworkIdV3>',
                    index: 'Compact<u64>'
                },
                AccountKey20: {
                    network: 'Option<NetworkIdV3>',
                    key: '[u8;20]'
                },
                PalletInstance: 'u8',
                GeneralIndex: 'Compact<u128>',
                GeneralKey: {
                    length: 'u8',
                    data: '[u8;32]'
                },
                OnlyChild: 'Null',
                Plurality: {
                    id: 'BodyIdV3',
                    part: 'BodyPartV3'
                },
                GlobalConsensus: 'NetworkIdV3'
            }
        },
        JunctionsV3: {
            _enum: {
                Here: 'Null',
                X1: 'JunctionV3',
                X2: '(JunctionV3, JunctionV3)',
                X3: '(JunctionV3, JunctionV3, JunctionV3)',
                X4: '(JunctionV3, JunctionV3, JunctionV3, JunctionV3)',
                X5: '(JunctionV3, JunctionV3, JunctionV3, JunctionV3, JunctionV3)',
                X6: '(JunctionV3, JunctionV3, JunctionV3, JunctionV3, JunctionV3, JunctionV3)',
                X7: '(JunctionV3, JunctionV3, JunctionV3, JunctionV3, JunctionV3, JunctionV3, JunctionV3)',
                X8: '(JunctionV3, JunctionV3, JunctionV3, JunctionV3, JunctionV3, JunctionV3, JunctionV3, JunctionV3)'
            }
        },
        MaybeErrorCodeV3: {
            _enum: {
                Success: 'Null',
                Error: 'Bytes',
                TruncatedError: 'Bytes'
            }
        },
        MaxPalletsInfo: 'u32',
        MultiAssetV3: {
            id: 'AssetIdV3',
            fun: 'FungibilityV3'
        },
        MultiAssetsV3: 'Vec<MultiAssetV3>',
        MultiAssetFilterV3: {
            _enum: {
                Definite: 'MultiAssetsV3',
                Wild: 'WildMultiAssetV3'
            }
        },
        MultiLocationV3: {
            parents: 'u8',
            interior: 'JunctionsV3'
        },
        MaxPalletNameLen: 'u32',
        NetworkIdV3: {
            _enum: {
                ByGenesis: '[u8;32]',
                ByFork: {
                    blockNumber: 'u64',
                    blockHash: '[u8;32]'
                },
                Polkadot: 'Null',
                Kusama: 'Null',
                Westend: 'Null',
                Rococo: 'Null',
                Wococo: 'Null',
                Ethereum: {
                    chainId: 'Compact<u64>'
                },
                BitcoinCore: 'Null',
                BitcoinCash: 'Null',
                PolkadotBulletin: 'Null'
            }
        },
        OriginKindV3: 'OriginKindV2',
        PalletInfoV3: {
            index: 'Compact<u32>',
            name: 'Bytes',
            moduleName: 'Bytes',
            major: 'Compact<u32>',
            minor: 'Compact<u32>',
            patch: 'Compact<u32>'
        },
        WildFungibilityV3: 'WildFungibilityV2',
        QueryResponseInfoV3: {
            destination: 'MultiLocationV3',
            queryId: 'Compact<u64>',
            maxWeight: 'WeightV2'
        },
        ResponseV3: {
            _enum: {
                Null: 'Null',
                Assets: 'MultiAssetsV3',
                ExecutionResult: 'Option<(u32,XcmErrorV3)>',
                Version: 'u32',
                PalletsInfo: 'Vec<PalletInfoV3>',
                DispatchResult: 'MaybeErrorCodeV3'
            }
        },
        XcmErrorV3: {
            _enum: {
                Overflow: 'Null',
                Unimplemented: 'Null',
                UntrustedReserveLocation: 'Null',
                UntrustedTeleportLocation: 'Null',
                LocationFull: 'Null',
                LocationNotInvertible: 'Null',
                BadOrigin: 'Null',
                InvalidLocation: 'Null',
                AssetNotFound: 'Null',
                FailedToTransactAsset: 'Null',
                NotWithdrawable: 'Null',
                LocationCannotHold: 'Null',
                ExceedsMaxMessageSize: 'Null',
                DestinationUnsupported: 'Null',
                Transport: 'Null',
                Unroutable: 'Null',
                UnknownClaim: 'Null',
                FailedToDecode: 'Null',
                MaxWeightInvalid: 'Null',
                NotHoldingFees: 'Null',
                TooExpensive: 'Null',
                Trap: 'u64',
                ExpectationFalse: 'Null',
                PalletNotFound: 'Null',
                NameMismatch: 'Null',
                VersionIncompatible: 'Null',
                HoldingWouldOverflow: 'Null',
                ExportError: 'Null',
                ReanchorFailed: 'Null',
                NoDeal: 'Null',
                FeesNotMet: 'Null',
                LockError: 'Null',
                NoPermission: 'Null',
                Unanchored: 'Null',
                NotDepositable: 'Null',
                UnhandledXcmVersion: 'Null',
                WeightLimitReached: 'WeightV2',
                Barrier: 'Null',
                ExceedsStackLimit: 'Null'
            }
        },
        ResponseV3Error: '(u32, XcmErrorV3)',
        ResponseV3Result: 'Option<(u32, ResponseV3Error)>',
        WeightLimitV3: {
            _enum: {
                Unlimited: 'Null',
                Limited: 'WeightV2'
            }
        },
        InstructionV3: {
            _enum: {
                WithdrawAsset: 'MultiAssetsV3',
                ReserveAssetDeposited: 'MultiAssetsV3',
                ReceiveTeleportedAsset: 'MultiAssetsV3',
                QueryResponse: {
                    queryId: 'Compact<u64>',
                    response: 'ResponseV3',
                    maxWeight: 'WeightV2',
                    querier: 'Option<MultiLocationV3>'
                },
                TransferAsset: {
                    assets: 'MultiAssetsV3',
                    beneficiary: 'MultiLocationV3'
                },
                TransferReserveAsset: {
                    assets: 'MultiAssetsV3',
                    dest: 'MultiLocationV3',
                    xcm: 'XcmV3'
                },
                Transact: {
                    originKind: 'XcmOriginKind',
                    requireWeightAtMost: 'WeightV2',
                    call: 'DoubleEncodedCall'
                },
                HrmpNewChannelOpenRequest: {
                    sender: 'Compact<u32>',
                    maxMessageSize: 'Compact<u32>',
                    maxCapacity: 'Compact<u32>'
                },
                HrmpChannelAccepted: {
                    recipient: 'Compact<u32>'
                },
                HrmpChannelClosing: {
                    initiator: 'Compact<u32>',
                    sender: 'Compact<u32>',
                    recipient: 'Compact<u32>'
                },
                ClearOrigin: 'Null',
                DescendOrigin: 'JunctionsV3',
                ReportError: 'QueryResponseInfoV3',
                DepositAsset: {
                    assets: 'MultiAssetFilterV3',
                    beneficiary: 'MultiLocationV3'
                },
                DepositReserveAsset: {
                    assets: 'MultiAssetFilterV3',
                    dest: 'MultiLocationV3',
                    xcm: 'XcmV3'
                },
                ExchangeAsset: {
                    give: 'MultiAssetFilterV3',
                    want: 'MultiAssetsV3',
                    maximal: 'bool'
                },
                InitiateReserveWithdraw: {
                    assets: 'MultiAssetFilterV3',
                    reserve: 'MultiLocationV3',
                    xcm: 'XcmV3'
                },
                InitiateTeleport: {
                    assets: 'MultiAssetFilterV3',
                    dest: 'MultiLocationV3',
                    xcm: 'XcmV3'
                },
                ReportHolding: {
                    responseInfo: 'QueryResponseInfoV3',
                    assets: 'MultiAssetFilterV3'
                },
                BuyExecution: {
                    fees: 'MultiAssetV3',
                    weightLimit: 'WeightLimitV3'
                },
                RefundSurplus: 'Null',
                SetErrorHandler: 'XcmV3<RuntimeCall>',
                SetAppendix: 'XcmV3<RuntimeCall>',
                ClearError: 'Null',
                ClaimAsset: {
                    assets: 'MultiAssetsV3',
                    ticket: 'MultiLocationV3'
                },
                Trap: 'Compact<u64>',
                SubscribeVersion: {
                    queryId: 'Compact<u64>',
                    maxResponseWeight: 'WeightV2'
                },
                UnsubscribeVersion: 'Null',
                BurnAsset: 'MultiAssetsV3',
                ExpectAsset: 'MultiAssetsV3',
                ExpectOrigin: 'Option<MultiLocationV3>',
                ExpectError: 'Option<(u32, XcmErrorV3)>',
                ExpectTransactStatus: 'MaybeErrorCodeV3',
                QueryPallet: {
                    moduleName: 'Vec<u8>',
                    responseInfo: 'QueryResponseInfoV3'
                },
                ExpectPallet: {
                    index: 'Compact<u32>',
                    name: 'Vec<u8>',
                    moduleName: 'Vec<u8>',
                    crateMajor: 'Compact<u32>',
                    minCrateMinor: 'Compact<u32>'
                },
                ReportTransactStatus: 'QueryResponseInfoV3',
                ClearTransactStatus: 'Null',
                UniversalOrigin: 'JunctionV3',
                ExportMessage: {
                    network: 'NetworkIdV3',
                    destination: 'JunctionsV3',
                    xcm: 'XcmV3'
                },
                LockAsset: {
                    asset: 'MultiAssetV3',
                    unlocker: 'MultiLocationV3'
                },
                UnlockAsset: {
                    asset: 'MultiAssetV3',
                    target: 'MultiLocationV3'
                },
                NoteUnlockable: {
                    asset: 'MultiAssetV3',
                    owner: 'MultiLocationV3'
                },
                RequestUnlock: {
                    asset: 'MultiAssetV3',
                    locker: 'MultiLocationV3'
                },
                SetFeesMode: {
                    jitWithdraw: 'bool'
                },
                SetTopic: '[u8; 32]',
                ClearTopic: 'Null',
                AliasOrigin: 'MultiLocationV3',
                UnpaidExecution: {
                    weightLimit: 'WeightLimitV3',
                    checkOrigin: 'Option<MultiLocationV3>'
                }
            }
        },
        WildMultiAssetV3: {
            _enum: {
                All: 'Null',
                AllOf: {
                    id: 'AssetIdV3',
                    fun: 'WildFungibilityV2'
                },
                AllCounted: 'Compact<u32>',
                AllOfCounted: {
                    id: 'AssetIdV3',
                    fun: 'WildFungibilityV2',
                    count: 'Compact<u32>'
                }
            }
        },
        VersionV3: 'u32',
        XcmV3: 'Vec<InstructionV3>'
    };

    const v4 = {
        AssetIdV4: 'MultiLocationV4',
        AssetInstanceV4: {
            _enum: {
                Undefined: 'Null',
                Index: 'Compact<u128>',
                Array4: '[u8;4]',
                Array8: '[u8;8]',
                Array16: '[u8;16]',
                Array32: '[u8;32]'
            }
        },
        FungibilityV4: {
            _enum: {
                Fungible: 'Compact<u128>',
                NonFungible: 'AssetInstanceV4'
            }
        },
        UncheckedFungibilityV4: {
            _enum: {
                Fungible: 'Compact<u128>',
                NonFungible: 'AssetInstanceV4'
            }
        },
        JunctionV4: {
            _enum: {
                Parachain: 'Compact<u32>',
                AccountId32: {
                    network: 'Option<NetworkIdV4>',
                    id: '[u8;32]'
                },
                AccountIndex64: {
                    network: 'Option<NetworkIdV4>',
                    index: 'Compact<u64>'
                },
                AccountKey20: {
                    network: 'Option<NetworkIdV4>',
                    key: '[u8;20]'
                },
                PalletInstance: 'u8',
                GeneralIndex: 'Compact<u128>',
                GeneralKey: {
                    length: 'u8',
                    data: '[u8;32]'
                },
                OnlyChild: 'Null',
                Plurality: {
                    id: 'BodyIdV3',
                    part: 'BodyPartV3'
                },
                GlobalConsensus: 'NetworkIdV4'
            }
        },
        JunctionsV4: {
            _enum: {
                Here: 'Null',
                X1: '[JunctionV4;1]',
                X2: '[JunctionV4;2]',
                X3: '[JunctionV4;3]',
                X4: '[JunctionV4;4]',
                X5: '[JunctionV4;5]',
                X6: '[JunctionV4;6]',
                X7: '[JunctionV4;7]',
                X8: '[JunctionV4;8]'
            }
        },
        MaxPalletsInfo: 'u32',
        NetworkIdV4: 'NetworkIdV3',
        MultiAssetV4: {
            id: 'AssetIdV4',
            fun: 'FungibilityV4'
        },
        MultiAssetsV4: 'Vec<MultiAssetV4>',
        MultiAssetFilterV4: {
            _enum: {
                Definite: 'MultiAssetsV4',
                Wild: 'WildMultiAssetV4'
            }
        },
        MultiLocationV4: {
            parents: 'u8',
            interior: 'JunctionsV4'
        },
        OriginKindV4: 'XcmOriginKind',
        PalletInfoV4: 'PalletInfoV3',
        WildFungibilityV4: 'WildFungibilityV2',
        QueryResponseInfoV4: {
            destination: 'MultiLocationV4',
            queryId: 'Compact<u64>',
            maxWeight: 'WeightV2'
        },
        ResponseV4: {
            _enum: {
                Null: 'Null',
                Assets: 'MultiAssetsV4',
                ExecutionResult: 'Option<(u32, XcmErrorV3)>',
                Version: 'u32',
                PalletsInfo: 'Vec<PalletInfoV4>',
                DispatchResult: 'MaybeErrorCodeV3'
            }
        },
        InstructionV4: {
            _enum: {
                WithdrawAsset: 'MultiAssetsV4',
                ReserveAssetDeposited: 'MultiAssetsV4',
                ReceiveTeleportedAsset: 'MultiAssetsV4',
                QueryResponse: {
                    queryId: 'Compact<u64>',
                    response: 'ResponseV4',
                    maxWeight: 'WeightV2',
                    querier: 'Option<MultiLocationV4>'
                },
                TransferAsset: {
                    assets: 'MultiAssetsV4',
                    beneficiary: 'MultiLocationV4'
                },
                TransferReserveAsset: {
                    assets: 'MultiAssetsV4',
                    dest: 'MultiLocationV4',
                    xcm: 'XcmV4'
                },
                Transact: {
                    originKind: 'OriginKindV4',
                    requireWeightAtMost: 'WeightV2',
                    call: 'DoubleEncodedCall'
                },
                HrmpNewChannelOpenRequest: {
                    sender: 'Compact<u32>',
                    maxMessageSize: 'Compact<u32>',
                    maxCapacity: 'Compact<u32>'
                },
                HrmpChannelAccepted: {
                    recipient: 'Compact<u32>'
                },
                HrmpChannelClosing: {
                    initiator: 'Compact<u32>',
                    sender: 'Compact<u32>',
                    recipient: 'Compact<u32>'
                },
                ClearOrigin: 'Null',
                DescendOrigin: 'JunctionsV4',
                ReportError: 'QueryResponseInfoV4',
                DepositAsset: {
                    assets: 'MultiAssetFilterV4',
                    beneficiary: 'MultiLocationV4'
                },
                DepositReserveAsset: {
                    assets: 'MultiAssetFilterV4',
                    dest: 'MultiLocationV4',
                    xcm: 'XcmV4'
                },
                ExchangeAsset: {
                    give: 'MultiAssetFilterV4',
                    want: 'MultiAssetsV4',
                    maximal: 'bool'
                },
                InitiateReserveWithdraw: {
                    assets: 'MultiAssetFilterV4',
                    reserve: 'MultiLocationV4',
                    xcm: 'XcmV4'
                },
                InitiateTeleport: {
                    assets: 'MultiAssetFilterV4',
                    dest: 'MultiLocationV4',
                    xcm: 'XcmV4'
                },
                ReportHolding: {
                    responseInfo: 'QueryResponseInfoV4',
                    assets: 'MultiAssetFilterV4'
                },
                BuyExecution: {
                    fees: 'MultiAssetV4',
                    weightLimit: 'WeightLimitV3'
                },
                RefundSurplus: 'Null',
                SetErrorHandler: 'XcmV4',
                SetAppendix: 'XcmV4',
                ClearError: 'Null',
                ClaimAsset: {
                    assets: 'MultiAssetsV4',
                    ticket: 'MultiLocationV4'
                },
                Trap: 'Compact<u64>',
                SubscribeVersion: {
                    queryId: 'Compact<u64>',
                    maxResponseWeight: 'WeightV2'
                },
                UnsubscribeVersion: 'Null',
                BurnAsset: 'MultiAssetsV4',
                ExpectAsset: 'MultiAssetsV4',
                ExpectOrigin: 'Option<MultiLocationV4>',
                ExpectError: 'Option<(u32, XcmErrorV3)>',
                ExpectTransactStatus: 'MaybeErrorCodeV3',
                QueryPallet: {
                    moduleName: 'Vec<u8>',
                    responseInfo: 'QueryResponseInfoV4'
                },
                ExpectPallet: {
                    index: 'Compact<u32>',
                    name: 'Vec<u8>',
                    moduleName: 'Vec<u8>',
                    crateMajor: 'Compact<u32>',
                    minCrateMinor: 'Compact<u32>'
                },
                ReportTransactStatus: 'QueryResponseInfoV4',
                ClearTransactStatus: 'Null',
                UniversalOrigin: 'JunctionV4',
                ExportMessage: {
                    network: 'NetworkIdV4',
                    destination: 'JunctionsV4',
                    xcm: 'XcmV4'
                },
                LockAsset: {
                    asset: 'MultiAssetV4',
                    unlocker: 'MultiLocationV4'
                },
                UnlockAsset: {
                    asset: 'MultiAssetV4',
                    target: 'MultiLocationV4'
                },
                NoteUnlockable: {
                    asset: 'MultiAssetV4',
                    owner: 'MultiLocationV4'
                },
                RequestUnlock: {
                    asset: 'MultiAssetV4',
                    locker: 'MultiLocationV4'
                },
                SetFeesMode: {
                    jitWithdraw: 'bool'
                },
                SetTopic: '[u8;32]',
                ClearTopic: 'Null',
                AliasOrigin: 'MultiLocationV4',
                UnpaidExecution: {
                    weightLimit: 'WeightLimitV3',
                    checkOrigin: 'Option<MultiLocationV4>'
                }
            }
        },
        OutcomeV4: {
            _enum: {
                Complete: {
                    used: 'WeightV2'
                },
                Incomplete: {
                    used: 'WeightV2',
                    error: 'XcmErrorV3'
                },
                Error: {
                    error: 'XcmErrorV3'
                }
            }
        },
        WildMultiAssetV4: {
            _enum: {
                All: 'Null',
                AllOf: {
                    id: 'AssetIdV4',
                    fun: 'WildFungibilityV4'
                },
                AllCounted: 'Compact<u32>',
                AllOfCounted: {
                    id: 'AssetIdV4',
                    fun: 'WildFungibilityV4',
                    count: 'Compact<u32>'
                }
            }
        },
        VersionV4: 'u32',
        XcmV4: 'Vec<InstructionV4>',
        XcmErrorV4: 'XcmErrorV3'
    };

    const v5 = {
        AssetIdV5: 'MultiLocationV5',
        AssetInstanceV5: {
            _enum: {
                Undefined: 'Null',
                Index: 'Compact<u128>',
                Array4: '[u8;4]',
                Array8: '[u8;8]',
                Array16: '[u8;16]',
                Array32: '[u8;32]'
            }
        },
        MultiAssetV5: {
            id: 'AssetIdV5',
            fun: 'FungibilityV5'
        },
        MultiAssetsV5: 'Vec<MultiAssetV5>',
        WildMultiAssetV5: {
            _enum: {
                All: 'Null',
                AllOf: {
                    id: 'AssetIdV5',
                    fun: 'WildFungibilityV5'
                },
                AllCounted: 'Compact<u32>',
                AllOfCounted: {
                    id: 'AssetIdV5',
                    fun: 'WildFungibilityV5',
                    count: 'Compact<u32>'
                }
            }
        },
        MultiAssetFilterV5: {
            _enum: {
                Definite: 'MultiAssetsV5',
                Wild: 'WildMultiAssetV5'
            }
        },
        MultiAssetTransferFilterV5: {
            _enum: {
                Teleport: 'MultiAssetFilterV5',
                ReserveDeposit: 'MultiAssetFilterV5',
                ReserveWithdraw: 'MultiAssetFilterV5'
            }
        },
        FungibilityV5: {
            _enum: {
                Fungible: 'Compact<u128>',
                NonFungible: 'AssetInstanceV5'
            }
        },
        UncheckedFungibilityV5: {
            _enum: {
                Fungible: 'Compact<u128>',
                NonFungible: 'AssetInstanceV5'
            }
        },
        WildFungibilityV5: 'WildFungibilityV2',
        JunctionV5: {
            _enum: {
                Parachain: 'Compact<u32>',
                AccountId32: {
                    network: 'Option<NetworkIdV5>',
                    id: '[u8;32]'
                },
                AccountIndex64: {
                    network: 'Option<NetworkIdV5>',
                    index: 'Compact<u64>'
                },
                AccountKey20: {
                    network: 'Option<NetworkIdV5>',
                    key: '[u8;20]'
                },
                PalletInstance: 'u8',
                GeneralIndex: 'Compact<u128>',
                GeneralKey: {
                    length: 'u8',
                    data: '[u8;32]'
                },
                OnlyChild: 'Null',
                Plurality: {
                    id: 'BodyIdV3',
                    part: 'BodyPartV3'
                },
                GlobalConsensus: 'NetworkIdV5'
            }
        },
        NetworkIdV5: {
            _enum: {
                ByGenesis: '[u8;32]',
                ByFork: {
                    blockNumber: 'u64',
                    blockHash: '[u8;32]'
                },
                Polkadot: 'Null',
                Kusama: 'Null',
                Ethereum: {
                    chainId: 'Compact<u64>'
                },
                BitcoinCore: 'Null',
                BitcoinCash: 'Null',
                PolkadotBulletin: 'Null'
            }
        },
        JunctionsV5: {
            _enum: {
                Here: 'Null',
                X1: '[JunctionV5;1]',
                X2: '[JunctionV5;2]',
                X3: '[JunctionV5;3]',
                X4: '[JunctionV5;4]',
                X5: '[JunctionV5;5]',
                X6: '[JunctionV5;6]',
                X7: '[JunctionV5;7]',
                X8: '[JunctionV5;8]'
            }
        },
        MultiLocationV5: {
            parents: 'u8',
            interior: 'JunctionsV5'
        },
        InteriorMultiLocationV5: 'JunctionsV5',
        XcmV5: 'Vec<InstructionV5>',
        PalletInfoV5: 'PalletInfoV3',
        ResponseV5: {
            _enum: {
                Null: 'Null',
                Assets: 'MultiAssetsV5',
                ExecutionResult: 'Option<(u32, XcmErrorV3)>',
                Version: 'u32',
                PalletsInfo: 'Vec<PalletInfoV5>',
                DispatchResult: 'MaybeErrorCodeV3'
            }
        },
        QueryResponseInfoV5: {
            destination: 'MultiLocationV5',
            queryId: 'Compact<u64>',
            maxWeight: 'WeightV2'
        },
        InstructionV5: {
            _enum: {
                WithdrawAsset: 'MultiAssetsV5',
                ReserveAssetDeposited: 'MultiAssetsV5',
                ReceiveTeleportedAsset: 'MultiAssetsV5',
                QueryResponse: {
                    queryId: 'Compact<u64>',
                    response: 'ResponseV5',
                    maxWeight: 'WeightV2',
                    querier: 'Option<MultiLocationV5>'
                },
                TransferAsset: {
                    assets: 'MultiAssetsV5',
                    beneficiary: 'MultiLocationV5'
                },
                TransferReserveAsset: {
                    assets: 'MultiAssetsV5',
                    dest: 'MultiLocationV5',
                    xcm: 'XcmV5'
                },
                Transact: {
                    originKind: 'OriginKindV4',
                    requireWeightAtMost: 'WeightV2',
                    call: 'DoubleEncodedCall'
                },
                HrmpNewChannelOpenRequest: {
                    sender: 'Compact<u32>',
                    maxMessageSize: 'Compact<u32>',
                    maxCapacity: 'Compact<u32>'
                },
                HrmpChannelAccepted: {
                    recipient: 'Compact<u32>'
                },
                HrmpChannelClosing: {
                    initiator: 'Compact<u32>',
                    sender: 'Compact<u32>',
                    recipient: 'Compact<u32>'
                },
                ClearOrigin: 'Null',
                DescendOrigin: 'InteriorMultiLocationV5',
                ReportError: 'QueryResponseInfoV4',
                DepositAsset: {
                    assets: 'MultiAssetFilterV5',
                    beneficiary: 'MultiLocationV5'
                },
                DepositReserveAsset: {
                    assets: 'MultiAssetFilterV5',
                    dest: 'MultiLocationV5',
                    xcm: 'XcmV5'
                },
                ExchangeAsset: {
                    give: 'MultiAssetFilterV5',
                    want: 'MultiAssetsV5',
                    maximal: 'bool'
                },
                InitiateReserveWithdraw: {
                    assets: 'MultiAssetFilterV5',
                    reserve: 'MultiLocationV5',
                    xcm: 'XcmV5'
                },
                InitiateTeleport: {
                    assets: 'MultiAssetFilterV5',
                    dest: 'MultiLocationV5',
                    xcm: 'XcmV5'
                },
                ReportHolding: {
                    responseInfo: 'QueryResponseInfoV4',
                    assets: 'MultiAssetFilterV5'
                },
                BuyExecution: {
                    fees: 'MultiAssetV4',
                    weightLimit: 'WeightLimitV3'
                },
                RefundSurplus: 'Null',
                SetErrorHandler: 'XcmV5',
                SetAppendix: 'XcmV5',
                ClearError: 'Null',
                ClaimAsset: {
                    assets: 'MultiAssetsV5',
                    ticket: 'MultiLocationV5'
                },
                Trap: 'Compact<u64>',
                SubscribeVersion: {
                    queryId: 'Compact<u64>',
                    maxResponseWeight: 'WeightV2'
                },
                UnsubscribeVersion: 'Null',
                BurnAsset: 'MultiAssetsV5',
                ExpectAsset: 'MultiAssetsV5',
                ExpectOrigin: 'Option<MultiLocationV5>',
                ExpectError: 'Option<(u32, XcmErrorV3)>',
                ExpectTransactStatus: 'MaybeErrorCodeV3',
                QueryPallet: {
                    moduleName: 'Vec<u8>',
                    responseInfo: 'QueryResponseInfoV4'
                },
                ExpectPallet: {
                    index: 'Compact<u32>',
                    name: 'Vec<u8>',
                    moduleName: 'Vec<u8>',
                    crateMajor: 'Compact<u32>',
                    minCrateMinor: 'Compact<u32>'
                },
                ReportTransactStatus: 'QueryResponseInfoV4',
                ClearTransactStatus: 'Null',
                UniversalOrigin: 'JunctionV5',
                ExportMessage: {
                    network: 'NetworkIdV5',
                    destination: 'InteriorMultiLocationV5',
                    xcm: 'XcmV5'
                },
                LockAsset: {
                    asset: 'MultiAssetV5',
                    unlocker: 'MultiLocationV5'
                },
                UnlockAsset: {
                    asset: 'MultiAssetV5',
                    target: 'MultiLocationV5'
                },
                NoteUnlockable: {
                    asset: 'MultiAssetV5',
                    owner: 'MultiLocationV5'
                },
                RequestUnlock: {
                    asset: 'MultiAssetV5',
                    locker: 'MultiLocationV5'
                },
                SetFeesMode: {
                    jitWithdraw: 'bool'
                },
                SetTopic: '[u8;32]',
                ClearTopic: 'Null',
                AliasOrigin: 'MultiLocationV5',
                UnpaidExecution: {
                    weightLimit: 'WeightLimitV3',
                    checkOrigin: 'Option<MultiLocationV5>'
                },
                PayFees: {
                    asset: 'MultiAssetV5'
                },
                InitiateTransfer: {
                    destination: 'MultiLocationV5',
                    remoteFees: 'Option<MultiAssetTransferFilterV5>',
                    preserveOrigin: 'bool',
                    assets: 'Vec<MultiAssetTransferFilterV5>',
                    remoteXcm: 'XcmV5'
                },
                ExecuteWithOrigin: {
                    descendant_origin: 'Option<InteriorMultiLocationV5>',
                    xcm: 'XcmV5'
                },
                SetHints: {
                    hints: 'Vec<HintV5>'
                }
            }
        },
        HintV5: {
            _enum: {
                AssetClaimer: {
                    location: 'MultiLocationV5'
                }
            }
        },
        XcmErrorV5: 'XcmErrorV3'
    };

    const XCM_LATEST = 'V5';
    const xcm = {
        XcmOrigin: {
            _enum: {
                Xcm: 'MultiLocation'
            }
        },
        XcmpMessageFormat: {
            _enum: ['ConcatenatedVersionedXcm', 'ConcatenatedEncodedBlob', 'Signals']
        },
        XcmAssetId: {
            _enum: {
                Concrete: 'MultiLocation',
                Abstract: 'Bytes'
            }
        },
        InboundStatus: {
            _enum: ['Ok', 'Suspended']
        },
        OutboundStatus: {
            _enum: ['Ok', 'Suspended']
        },
        MultiAssets: 'Vec<MultiAsset>'
    };
    const location = {
        BodyId: {
            _enum: {
                Unit: 'Null',
                Named: 'Vec<u8>',
                Index: 'Compact<u32>',
                Executive: 'Null',
                Technical: 'Null',
                Legislative: 'Null',
                Judicial: 'Null'
            }
        },
        BodyPart: {
            _enum: {
                Voice: 'Null',
                Members: 'Compact<u32>',
                Fraction: {
                    nom: 'Compact<u32>',
                    denom: 'Compact<u32>'
                },
                AtLeastProportion: {
                    nom: 'Compact<u32>',
                    denom: 'Compact<u32>'
                },
                MoreThanProportion: {
                    nom: 'Compact<u32>',
                    denom: 'Compact<u32>'
                }
            }
        },
        InteriorMultiLocation: 'Junctions',
        NetworkId: {
            _enum: {
                Any: 'Null',
                Named: 'Vec<u8>',
                Polkadot: 'Null',
                Kusama: 'Null'
            }
        }
    };
    const definitions$f = {
        rpc: {},
        types: {
            ...location,
            ...xcm,
            ...v0,
            ...v1,
            ...v2,
            ...v3,
            ...v4,
            ...v5,
            ...mapXcmTypes(XCM_LATEST),
            DoubleEncodedCall: {
                encoded: 'Bytes'
            },
            XcmOriginKind: {
                _enum: ['Native', 'SovereignAccount', 'Superuser', 'Xcm']
            },
            Outcome: {
                _enum: {
                    Complete: 'Weight',
                    Incomplete: '(Weight, XcmErrorV0)',
                    Error: 'XcmErrorV0'
                }
            },
            QueryId: 'u64',
            QueryStatus: {
                _enum: {
                    Pending: {
                        responder: 'VersionedMultiLocation',
                        maybeNotify: 'Option<(u8, u8)>',
                        timeout: 'BlockNumber'
                    },
                    Ready: {
                        response: 'VersionedResponse',
                        at: 'BlockNumber'
                    }
                }
            },
            QueueConfigData: {
                suspendThreshold: 'u32',
                dropThreshold: 'u32',
                resumeThreshold: 'u32',
                thresholdWeight: 'Weight',
                weightRestrictDecay: 'Weight'
            },
            VersionMigrationStage: {
                _enum: {
                    MigrateSupportedVersion: 'Null',
                    MigrateVersionNotifiers: 'Null',
                    NotifyCurrentTargets: 'Option<Bytes>',
                    MigrateAndNotifyOldTargets: 'Null'
                }
            },
            VersionedMultiAsset: {
                _enum: {
                    V0: 'MultiAssetV0',
                    V1: 'MultiAssetV1',
                    V2: 'MultiAssetV2',
                    V3: 'MultiAssetV3',
                    V4: 'MultiAssetV4',
                    V5: 'MultiAssetV5'
                }
            },
            VersionedMultiAssets: {
                _enum: {
                    V0: 'Vec<MultiAssetV0>',
                    V1: 'MultiAssetsV1',
                    V2: 'MultiAssetsV2',
                    V3: 'MultiAssetsV3',
                    V4: 'MultiAssetsV4',
                    V5: 'MultiAssetsV5'
                }
            },
            VersionedMultiLocation: {
                _enum: {
                    V0: 'MultiLocationV0',
                    V1: 'MultiLocationV1',
                    V2: 'MultiLocationV2',
                    V3: 'MultiLocationV3',
                    V4: 'MultiLocationV4',
                    v5: 'MultiLocationV5'
                }
            },
            VersionedResponse: {
                V0: 'ResponseV0',
                V1: 'ResponseV1',
                V2: 'ResponseV2',
                V3: 'ResponseV3',
                V4: 'ResponseV4',
                V5: 'ResponseV5'
            },
            VersionedXcm: {
                _enum: {
                    V0: 'XcmV0',
                    V1: 'XcmV1',
                    V2: 'XcmV2',
                    V3: 'XcmV3',
                    V4: 'XcmV4',
                    V5: 'XcmV5'
                }
            },
            XcmVersion: 'u32'
        }
    };

    const runtime$7 = {
        XcmPaymentApi: [
            {
                methods: {
                    query_acceptable_payment_assets: {
                        description: 'The API to query acceptable payment assets',
                        params: [
                            {
                                name: 'version',
                                type: 'u32'
                            }
                        ],
                        type: 'Result<Vec<XcmVersionedAssetId>, XcmPaymentApiError>'
                    },
                    query_weight_to_asset_fee: {
                        description: '',
                        params: [
                            {
                                name: 'weight',
                                type: 'WeightV2'
                            },
                            {
                                name: 'asset',
                                type: 'XcmVersionedAssetId'
                            }
                        ],
                        type: 'Result<u128, XcmPaymentApiError>'
                    },
                    query_xcm_weight: {
                        description: '',
                        params: [
                            {
                                name: 'message',
                                type: 'XcmVersionedXcm'
                            }
                        ],
                        type: 'Result<WeightV2, XcmPaymentApiError>'
                    }
                },
                version: 1
            }
        ]
    };

    const definitions$e = {
        rpc: {},
        runtime: runtime$7,
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

    const runtime$6 = {
        LocationToAccountApi: [
            {
                methods: {
                    convert_location: {
                        description: 'Converts `Location` to `AccountId`',
                        params: [
                            {
                                name: 'location',
                                type: 'XcmVersionedLocation'
                            }
                        ],
                        type: 'Result<AccountId, Error>'
                    }
                },
                version: 1
            }
        ]
    };

    const definitions$d = {
        rpc: {},
        runtime: runtime$6,
        types: {
            Error: {
                _enum: ['Unsupported', 'VersionedConversionFailed']
            }
        }
    };

    const layout = {
        ContractCryptoHasher: {
            _enum: ['Blake2x256', 'Sha2x256', 'Keccak256']
        },
        ContractDiscriminant: 'u32',
        ContractLayoutArray: {
            offset: 'ContractLayoutKey',
            len: 'u32',
            cellsPerElem: 'u64',
            layout: 'ContractStorageLayout'
        },
        ContractLayoutCell: {
            key: 'ContractLayoutKey',
            ty: 'SiLookupTypeId'
        },
        ContractLayoutEnum: {
            dispatchKey: 'ContractLayoutKey',
            variants: 'BTreeMap<ContractDiscriminant, ContractLayoutStruct>'
        },
        ContractLayoutHash: {
            offset: 'ContractLayoutKey',
            strategy: 'ContractLayoutHashingStrategy',
            layout: 'ContractStorageLayout'
        },
        ContractLayoutHashingStrategy: {
            hasher: 'ContractCryptoHasher',
            postfix: 'Vec<u8>',
            prefix: 'Vec<u8>'
        },
        ContractLayoutKey: '[u8; 32]',
        ContractLayoutStruct: {
            fields: 'Vec<ContractLayoutStructField>'
        },
        ContractLayoutStructField: {
            layout: 'ContractStorageLayout',
            name: 'Text'
        },
        ContractStorageLayout: {
            _enum: {
                Cell: 'ContractLayoutCell',
                Hash: 'ContractLayoutHash',
                Array: 'ContractLayoutArray',
                Struct: 'ContractLayoutStruct',
                Enum: 'ContractLayoutEnum'
            }
        }
    };
    const spec = {
        ContractConstructorSpecV0: {
            name: 'Text',
            selector: 'ContractSelector',
            args: 'Vec<ContractMessageParamSpecV0>',
            docs: 'Vec<Text>'
        },
        ContractConstructorSpecV1: {
            name: 'Vec<Text>',
            selector: 'ContractSelector',
            args: 'Vec<ContractMessageParamSpecV0>',
            docs: 'Vec<Text>'
        },
        ContractConstructorSpecV2: {
            label: 'Text',
            selector: 'ContractSelector',
            args: 'Vec<ContractMessageParamSpecV2>',
            docs: 'Vec<Text>'
        },
        ContractConstructorSpecV3: {
            label: 'Text',
            selector: 'ContractSelector',
            payable: 'bool',
            args: 'Vec<ContractMessageParamSpecV2>',
            docs: 'Vec<Text>'
        },
        ContractConstructorSpecV4: {
            label: 'Text',
            selector: 'ContractSelector',
            payable: 'bool',
            args: 'Vec<ContractMessageParamSpecV2>',
            docs: 'Vec<Text>',
            default: 'bool',
            returnType: 'Option<ContractTypeSpec>'
        },
        ContractContractSpecV0: {
            constructors: 'Vec<ContractConstructorSpecV0>',
            messages: 'Vec<ContractMessageSpecV0>',
            events: 'Vec<ContractEventSpecV0>',
            docs: 'Vec<Text>'
        },
        ContractContractSpecV1: {
            constructors: 'Vec<ContractConstructorSpecV1>',
            messages: 'Vec<ContractMessageSpecV1>',
            events: 'Vec<ContractEventSpecV1>',
            docs: 'Vec<Text>'
        },
        ContractContractSpecV2: {
            constructors: 'Vec<ContractConstructorSpecV2>',
            messages: 'Vec<ContractMessageSpecV2>',
            events: 'Vec<ContractEventSpecV2>',
            docs: 'Vec<Text>'
        },
        ContractContractSpecV3: {
            constructors: 'Vec<ContractConstructorSpecV3>',
            messages: 'Vec<ContractMessageSpecV2>',
            events: 'Vec<ContractEventSpecV2>',
            docs: 'Vec<Text>'
        },
        ContractContractSpecV4: {
            constructors: 'Vec<ContractConstructorSpecV4>',
            messages: 'Vec<ContractMessageSpecV3>',
            events: 'Vec<ContractEventSpecV2>',
            docs: 'Vec<Text>',
            environment: 'ContractEnvironmentV4'
        },
        ContractContractSpecV5: {
            constructors: 'Vec<ContractConstructorSpecV4>',
            messages: 'Vec<ContractMessageSpecV3>',
            events: 'Vec<ContractEventSpecV3>',
            docs: 'Vec<Text>',
            environment: 'ContractEnvironmentV4'
        },
        ContractContractSpecV6: {
            constructors: 'Vec<ContractConstructorSpecV4>',
            messages: 'Vec<ContractMessageSpecV3>',
            events: 'Vec<ContractEventSpecV3>',
            docs: 'Vec<Text>',
            environment: 'ContractEnvironmentV4'
        },
        ContractDisplayName: 'SiPath',
        ContractEventParamSpecV0: {
            name: 'Text',
            indexed: 'bool',
            type: 'ContractTypeSpec',
            docs: 'Vec<Text>'
        },
        ContractEventParamSpecV2: {
            label: 'Text',
            indexed: 'bool',
            type: 'ContractTypeSpec',
            docs: 'Vec<Text>'
        },
        ContractEventSpecV0: {
            name: 'Text',
            args: 'Vec<ContractEventParamSpecV0>',
            docs: 'Vec<Text>'
        },
        ContractEventSpecV1: {
            name: 'Text',
            args: 'Vec<ContractEventParamSpecV0>',
            docs: 'Vec<Text>'
        },
        ContractEventSpecV2: {
            label: 'Text',
            args: 'Vec<ContractEventParamSpecV2>',
            docs: 'Vec<Text>'
        },
        ContractEventSpecV3: {
            label: 'Text',
            args: 'Vec<ContractEventParamSpecV2>',
            docs: 'Vec<Text>',
            module_path: 'Text',
            signature_topic: 'Option<[u8; 32]>'
        },
        ContractMessageParamSpecV0: {
            name: 'Text',
            type: 'ContractTypeSpec'
        },
        ContractMessageParamSpecV2: {
            label: 'Text',
            type: 'ContractTypeSpec'
        },
        ContractMessageSpecV0: {
            name: 'Text',
            selector: 'ContractSelector',
            mutates: 'bool',
            payable: 'bool',
            args: 'Vec<ContractMessageParamSpecV0>',
            returnType: 'Option<ContractTypeSpec>',
            docs: 'Vec<Text>'
        },
        ContractMessageSpecV1: {
            name: 'Vec<Text>',
            selector: 'ContractSelector',
            mutates: 'bool',
            payable: 'bool',
            args: 'Vec<ContractMessageParamSpecV0>',
            returnType: 'Option<ContractTypeSpec>',
            docs: 'Vec<Text>'
        },
        ContractMessageSpecV2: {
            label: 'Text',
            selector: 'ContractSelector',
            mutates: 'bool',
            payable: 'bool',
            args: 'Vec<ContractMessageParamSpecV2>',
            returnType: 'Option<ContractTypeSpec>',
            docs: 'Vec<Text>'
        },
        ContractMessageSpecV3: {
            label: 'Text',
            selector: 'ContractSelector',
            mutates: 'bool',
            payable: 'bool',
            args: 'Vec<ContractMessageParamSpecV2>',
            returnType: 'Option<ContractTypeSpec>',
            docs: 'Vec<Text>',
            default: 'bool'
        },
        ContractSelector: '[u8; 4]',
        ContractTypeSpec: {
            type: 'SiLookupTypeId',
            displayName: 'ContractDisplayName'
        }
    };
    const latest = {
        ContractConstructorSpecLatest: 'ContractConstructorSpecV4',
        ContractEventSpecLatest: 'ContractEventSpecV3',
        ContractEventParamSpecLatest: 'ContractEventParamSpecV2',
        ContractMessageParamSpecLatest: 'ContractMessageParamSpecV2',
        ContractMessageSpecLatest: 'ContractMessageSpecV3',
        ContractMetadataLatest: 'ContractMetadataV6'
    };
    const definitions$c = {
        rpc: {},
        types: {
            ...layout,
            ...spec,
            ...latest,
            ContractProjectInfo: {
                source: 'ContractProjectSource',
                contract: 'ContractProjectContract'
            },
            ContractReviveProjectInfo: {
                source: 'ContractReviveProjectSource',
                contract: 'ContractProjectContract'
            },
            ContractMetadataV0: {
                metadataVersion: 'Text',
                types: 'Vec<Si0Type>',
                spec: 'ContractContractSpecV0'
            },
            ContractMetadataV1: {
                types: 'Vec<PortableType>',
                spec: 'ContractContractSpecV1'
            },
            ContractMetadataV2: {
                types: 'Vec<PortableType>',
                spec: 'ContractContractSpecV2'
            },
            ContractMetadataV3: {
                types: 'Vec<PortableType>',
                spec: 'ContractContractSpecV3'
            },
            ContractMetadataV4: {
                types: 'Vec<PortableType>',
                spec: 'ContractContractSpecV4',
                version: 'Text'
            },
            ContractMetadataV5: {
                types: 'Vec<PortableType>',
                spec: 'ContractContractSpecV5',
                version: 'u64'
            },
            ContractMetadataV6: {
                types: 'Vec<PortableType>',
                spec: 'ContractContractSpecV6',
                version: 'u64'
            },
            ContractMetadata: {
                _enum: {
                    V0: 'ContractMetadataV0',
                    V1: 'ContractMetadataV1',
                    V2: 'ContractMetadataV2',
                    V3: 'ContractMetadataV3',
                    V4: 'ContractMetadataV4',
                    V5: 'ContractMetadataV5',
                    V6: 'ContractMetadataV6'
                }
            },
            ContractProjectV0: {
                metadataVersion: 'Text',
                source: 'ContractProjectSource',
                contract: 'ContractProjectContract',
                types: 'Vec<Si0Type>',
                spec: 'ContractContractSpecV0'
            },
            ContractProject: '(ContractProjectInfo, ContractMetadata)',
            ContractProjectContract: {
                _alias: {
                    docs: 'documentation'
                },
                name: 'Text',
                version: 'Text',
                authors: 'Vec<Text>',
                description: 'Option<Text>',
                docs: 'Option<Text>',
                repository: 'Option<Text>',
                homepage: 'Option<Text>',
                license: 'Option<Text>'
            },
            ContractProjectSource: {
                _alias: {
                    wasmHash: 'hash'
                },
                wasmHash: '[u8; 32]',
                language: 'Text',
                compiler: 'Text',
                wasm: 'Raw'
            },
            ContractReviveProjectSource: {
                _alias: {
                    wasmHash: 'hash',
                    wasm: 'contract_binary'
                },
                wasmHash: '[u8; 32]',
                language: 'Text',
                compiler: 'Text',
                wasm: 'Raw'
            },
            ContractEnvironmentV4: {
                _alias: {
                    hashType: 'hash'
                },
                accountId: 'Option<ContractTypeSpec>',
                balance: 'Option<ContractTypeSpec>',
                blockNumber: 'Option<ContractTypeSpec>',
                hashType: 'Option<ContractTypeSpec>',
                timestamp: 'Option<ContractTypeSpec>',
                maxEventTopics: 'Option<u32>'
            }
        }
    };

    const netRpc = {
        listening: {
            aliasSection: 'net',
            description: 'Returns true if client is actively listening for network connections. Otherwise false.',
            params: [],
            type: 'bool'
        },
        peerCount: {
            aliasSection: 'net',
            description: 'Returns number of peers connected to node.',
            params: [],
            type: 'Text'
        },
        version: {
            aliasSection: 'net',
            description: 'Returns protocol version.',
            params: [],
            type: 'Text'
        }
    };
    const web3Rpc = {
        clientVersion: {
            aliasSection: 'web3',
            description: 'Returns current client version.',
            params: [],
            type: 'Text'
        },
        sha3: {
            aliasSection: 'web3',
            description: 'Returns sha3 of the given data',
            params: [{ name: 'data', type: 'Bytes' }],
            type: 'H256'
        }
    };
    const rpc$7 = {
        ...netRpc,
        ...web3Rpc,
        accounts: {
            description: 'Returns accounts list.',
            params: [],
            type: 'Vec<H160>'
        },
        blockNumber: {
            description: 'Returns the blockNumber',
            params: [],
            type: 'U256'
        },
        call: {
            description: 'Call contract, returning the output data.',
            params: [
                {
                    name: 'request',
                    type: 'EthCallRequest'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'number',
                    type: 'BlockNumber'
                }
            ],
            type: 'Bytes'
        },
        chainId: {
            description: 'Returns the chain ID used for transaction signing at the current best block. None is returned if not available.',
            params: [],
            type: 'U64'
        },
        coinbase: {
            description: 'Returns block author.',
            params: [],
            type: 'H160'
        },
        estimateGas: {
            description: 'Estimate gas needed for execution of given contract.',
            params: [
                {
                    name: 'request',
                    type: 'EthCallRequest'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'number',
                    type: 'BlockNumber'
                }
            ],
            type: 'U256'
        },
        feeHistory: {
            description: 'Returns fee history for given block count & reward percentiles',
            params: [
                {
                    name: 'blockCount',
                    type: 'U256'
                },
                {
                    name: 'newestBlock',
                    type: 'BlockNumber'
                },
                {
                    name: 'rewardPercentiles',
                    type: 'Option<Vec<f64>>'
                }
            ],
            type: 'EthFeeHistory'
        },
        gasPrice: {
            description: 'Returns current gas price.',
            params: [],
            type: 'U256'
        },
        getBalance: {
            description: 'Returns balance of the given account.',
            params: [
                {
                    name: 'address',
                    type: 'H160'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'number',
                    type: 'BlockNumber'
                }
            ],
            type: 'U256'
        },
        getBlockByHash: {
            description: 'Returns block with given hash.',
            params: [
                {
                    name: 'hash',
                    type: 'H256'
                },
                {
                    name: 'full',
                    type: 'bool'
                }
            ],
            type: 'Option<EthRichBlock>'
        },
        getBlockByNumber: {
            description: 'Returns block with given number.',
            params: [
                {
                    name: 'block',
                    type: 'BlockNumber'
                },
                { name: 'full', type: 'bool' }
            ],
            type: 'Option<EthRichBlock>'
        },
        getBlockTransactionCountByHash: {
            description: 'Returns the number of transactions in a block with given hash.',
            params: [
                {
                    name: 'hash',
                    type: 'H256'
                }
            ],
            type: 'U256'
        },
        getBlockTransactionCountByNumber: {
            description: 'Returns the number of transactions in a block with given block number.',
            params: [
                {
                    name: 'block',
                    type: 'BlockNumber'
                }
            ],
            type: 'U256'
        },
        getCode: {
            description: 'Returns the code at given address at given time (block number).',
            params: [
                {
                    name: 'address',
                    type: 'H160'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'number',
                    type: 'BlockNumber'
                }
            ],
            type: 'Bytes'
        },
        getFilterChanges: {
            description: 'Returns filter changes since last poll.',
            params: [
                {
                    name: 'index',
                    type: 'U256'
                }
            ],
            type: 'EthFilterChanges'
        },
        getFilterLogs: {
            description: 'Returns all logs matching given filter (in a range \'from\' - \'to\').',
            params: [
                {
                    name: 'index',
                    type: 'U256'
                }
            ],
            type: 'Vec<EthLog>'
        },
        getLogs: {
            description: 'Returns logs matching given filter object.',
            params: [
                {
                    name: 'filter',
                    type: 'EthFilter'
                }
            ],
            type: 'Vec<EthLog>'
        },
        getProof: {
            description: 'Returns proof for account and storage.',
            params: [
                {
                    name: 'address',
                    type: 'H160'
                },
                {
                    name: 'storageKeys',
                    type: 'Vec<H256>'
                },
                {
                    name: 'number',
                    type: 'BlockNumber'
                }
            ],
            type: 'EthAccount'
        },
        getStorageAt: {
            description: 'Returns content of the storage at given address.',
            params: [
                {
                    name: 'address',
                    type: 'H160'
                },
                {
                    name: 'index',
                    type: 'U256'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'number',
                    type: 'BlockNumber'
                }
            ],
            type: 'H256'
        },
        getTransactionByBlockHashAndIndex: {
            description: 'Returns transaction at given block hash and index.',
            params: [
                {
                    name: 'hash',
                    type: 'H256'
                },
                {
                    name: 'index',
                    type: 'U256'
                }
            ],
            type: 'EthTransaction'
        },
        getTransactionByBlockNumberAndIndex: {
            description: 'Returns transaction by given block number and index.',
            params: [
                {
                    name: 'number',
                    type: 'BlockNumber'
                },
                {
                    name: 'index',
                    type: 'U256'
                }
            ],
            type: 'EthTransaction'
        },
        getTransactionByHash: {
            description: 'Get transaction by its hash.',
            params: [
                {
                    name: 'hash',
                    type: 'H256'
                }
            ],
            type: 'EthTransaction'
        },
        getTransactionCount: {
            description: 'Returns the number of transactions sent from given address at given time (block number).',
            params: [
                {
                    name: 'address',
                    type: 'H160'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'number',
                    type: 'BlockNumber'
                }
            ],
            type: 'U256'
        },
        getTransactionReceipt: {
            description: 'Returns transaction receipt by transaction hash.',
            params: [
                {
                    name: 'hash',
                    type: 'H256'
                }
            ],
            type: 'EthReceipt'
        },
        getUncleByBlockHashAndIndex: {
            description: 'Returns an uncles at given block and index.',
            params: [
                {
                    name: 'hash',
                    type: 'H256'
                },
                {
                    name: 'index',
                    type: 'U256'
                }
            ],
            type: 'EthRichBlock'
        },
        getUncleByBlockNumberAndIndex: {
            description: 'Returns an uncles at given block and index.',
            params: [
                {
                    name: 'number',
                    type: 'BlockNumber'
                },
                {
                    name: 'index',
                    type: 'U256'
                }
            ],
            type: 'EthRichBlock'
        },
        getUncleCountByBlockHash: {
            description: 'Returns the number of uncles in a block with given hash.',
            params: [
                {
                    name: 'hash',
                    type: 'H256'
                }
            ],
            type: 'U256'
        },
        getUncleCountByBlockNumber: {
            description: 'Returns the number of uncles in a block with given block number.',
            params: [
                {
                    name: 'number',
                    type: 'BlockNumber'
                }
            ],
            type: 'U256'
        },
        getWork: {
            description: 'Returns the hash of the current block, the seedHash, and the boundary condition to be met.',
            params: [],
            type: 'EthWork'
        },
        hashrate: {
            description: 'Returns the number of hashes per second that the node is mining with.',
            params: [],
            type: 'U256'
        },
        maxPriorityFeePerGas: {
            description: 'Returns max priority fee per gas',
            params: [],
            type: 'U256'
        },
        mining: {
            description: 'Returns true if client is actively mining new blocks.',
            params: [],
            type: 'bool'
        },
        newBlockFilter: {
            description: 'Returns id of new block filter.',
            params: [],
            type: 'U256'
        },
        newFilter: {
            description: 'Returns id of new filter.',
            params: [
                {
                    name: 'filter',
                    type: 'EthFilter'
                }
            ],
            type: 'U256'
        },
        newPendingTransactionFilter: {
            description: 'Returns id of new block filter.',
            params: [],
            type: 'U256'
        },
        protocolVersion: {
            description: 'Returns protocol version encoded as a string (quotes are necessary).',
            params: [],
            type: 'u64'
        },
        sendRawTransaction: {
            description: 'Sends signed transaction, returning its hash.',
            params: [
                {
                    name: 'bytes',
                    type: 'Bytes'
                }
            ],
            type: 'H256'
        },
        sendTransaction: {
            description: 'Sends transaction; will block waiting for signer to return the transaction hash',
            params: [
                {
                    name: 'tx',
                    type: 'EthTransactionRequest'
                }
            ],
            type: 'H256'
        },
        submitHashrate: {
            description: 'Used for submitting mining hashrate.',
            params: [
                {
                    name: 'index',
                    type: 'U256'
                },
                {
                    name: 'hash',
                    type: 'H256'
                }
            ],
            type: 'bool'
        },
        submitWork: {
            description: 'Used for submitting a proof-of-work solution.',
            params: [
                {
                    name: 'nonce',
                    type: 'H64'
                },
                {
                    name: 'headerHash',
                    type: 'H256'
                },
                {
                    name: 'mixDigest',
                    type: 'H256'
                }
            ],
            type: 'bool'
        },
        subscribe: {
            description: 'Subscribe to Eth subscription.',
            params: [
                { name: 'kind', type: 'EthSubKind' },
                {
                    isOptional: true,
                    name: 'params',
                    type: 'EthSubParams'
                }
            ],
            pubsub: [
                'subscription',
                'subscribe',
                'unsubscribe'
            ],
            type: 'Null'
        },
        syncing: {
            description: 'Returns an object with data about the sync status or false.',
            params: [],
            type: 'EthSyncStatus'
        },
        uninstallFilter: {
            description: 'Uninstalls filter.',
            params: [
                {
                    name: 'index',
                    type: 'U256'
                }
            ],
            type: 'bool'
        }
    };

    const ethMethodsV4 = {
        account_basic: {
            description: 'Returns pallet_evm::Accounts by address.',
            params: [
                {
                    name: 'address',
                    type: 'H160'
                }
            ],
            type: 'EvmAccount'
        },
        account_code_at: {
            description: 'For a given account address, returns pallet_evm::AccountCodes.',
            params: [
                {
                    name: 'address',
                    type: 'H160'
                }
            ],
            type: 'Bytes'
        },
        author: {
            description: 'Returns the converted FindAuthor::find_author authority id.',
            params: [],
            type: 'H160'
        },
        call: {
            description: 'Returns a frame_ethereum::call response. If `estimate` is true,',
            params: [
                {
                    name: 'from',
                    type: 'H160'
                },
                {
                    name: 'to',
                    type: 'H160'
                },
                {
                    name: 'data',
                    type: 'Vec<u8>'
                },
                {
                    name: 'value',
                    type: 'U256'
                },
                {
                    name: 'gasLimit',
                    type: 'U256'
                },
                {
                    name: 'maxFeePerGas',
                    type: 'Option<U256>'
                },
                {
                    name: 'maxPriorityFeePerGas',
                    type: 'Option<U256>'
                },
                {
                    name: 'nonce',
                    type: 'Option<U256>'
                },
                {
                    name: 'estimate',
                    type: 'bool'
                },
                {
                    name: 'accessList',
                    type: 'Option<Vec<(H160, Vec<H256>)>>'
                }
            ],
            type: 'Result<EvmCallInfo, DispatchError>'
        },
        chain_id: {
            description: 'Returns runtime defined pallet_evm::ChainId.',
            params: [],
            type: 'u64'
        },
        create: {
            description: 'Returns a frame_ethereum::call response. If `estimate` is true,',
            params: [
                {
                    name: 'from',
                    type: 'H160'
                },
                {
                    name: 'data',
                    type: 'Vec<u8>'
                },
                {
                    name: 'value',
                    type: 'U256'
                },
                {
                    name: 'gasLimit',
                    type: 'U256'
                },
                {
                    name: 'maxFeePerGas',
                    type: 'Option<U256>'
                },
                {
                    name: 'maxPriorityFeePerGas',
                    type: 'Option<U256>'
                },
                {
                    name: 'nonce',
                    type: 'Option<U256>'
                },
                {
                    name: 'estimate',
                    type: 'bool'
                },
                {
                    name: 'accessList',
                    type: 'Option<Vec<(H160, Vec<H256>)>>'
                }
            ],
            type: 'Result<EvmCreateInfo, DispatchError>'
        },
        current_all: {
            description: 'Return all the current data for a block in a single runtime call.',
            params: [],
            type: '(Option<BlockV2>, Option<Vec<EthReceiptV3>>, Option<Vec<EthTransactionStatus>>)'
        },
        current_block: {
            description: 'Return the current block.',
            params: [],
            type: 'BlockV2'
        },
        current_receipts: {
            description: 'Return the current receipt.',
            params: [],
            type: 'Option<Vec<EthReceiptV3>>'
        },
        current_transaction_statuses: {
            description: 'Return the current transaction status.',
            params: [],
            type: 'Option<Vec<EthTransactionStatus>>'
        },
        elasticity: {
            description: 'Return the elasticity multiplier.',
            params: [],
            type: 'Option<Permill>'
        },
        extrinsic_filter: {
            description: 'Receives a `Vec<OpaqueExtrinsic>` and filters all the ethereum transactions.',
            params: [
                {
                    name: 'xts',
                    type: 'Vec<Extrinsic>'
                }
            ],
            type: 'Vec<TransactionV2>'
        },
        gas_price: {
            description: 'Returns FixedGasPrice::min_gas_price',
            params: [],
            type: 'u256'
        },
        storage_at: {
            description: 'For a given account address and index, returns pallet_evm::AccountStorages.',
            params: [
                {
                    name: 'address',
                    type: 'H160'
                },
                {
                    name: 'index',
                    type: 'u256'
                }
            ],
            type: 'H256'
        }
    };
    const ethMethodsV5 = {
        call: {
            description: 'Returns a frame_ethereum::call response. If `estimate` is true,',
            params: [
                {
                    name: 'from',
                    type: 'H160'
                },
                {
                    name: 'to',
                    type: 'H160'
                },
                {
                    name: 'data',
                    type: 'Vec<u8>'
                },
                {
                    name: 'value',
                    type: 'U256'
                },
                {
                    name: 'gasLimit',
                    type: 'U256'
                },
                {
                    name: 'maxFeePerGas',
                    type: 'Option<U256>'
                },
                {
                    name: 'maxPriorityFeePerGas',
                    type: 'Option<U256>'
                },
                {
                    name: 'nonce',
                    type: 'Option<U256>'
                },
                {
                    name: 'estimate',
                    type: 'bool'
                },
                {
                    name: 'accessList',
                    type: 'Option<Vec<(H160, Vec<H256>)>>'
                }
            ],
            type: 'Result<EvmCallInfoV2, DispatchError>'
        },
        create: {
            description: 'Returns a frame_ethereum::call response. If `estimate` is true,',
            params: [
                {
                    name: 'from',
                    type: 'H160'
                },
                {
                    name: 'data',
                    type: 'Vec<u8>'
                },
                {
                    name: 'value',
                    type: 'U256'
                },
                {
                    name: 'gasLimit',
                    type: 'U256'
                },
                {
                    name: 'maxFeePerGas',
                    type: 'Option<U256>'
                },
                {
                    name: 'maxPriorityFeePerGas',
                    type: 'Option<U256>'
                },
                {
                    name: 'nonce',
                    type: 'Option<U256>'
                },
                {
                    name: 'estimate',
                    type: 'bool'
                },
                {
                    name: 'accessList',
                    type: 'Option<Vec<(H160, Vec<H256>)>>'
                }
            ],
            type: 'Result<EvmCreateInfoV2, DispatchError>'
        }
    };
    const runtime$5 = {
        ConvertTransactionRuntimeApi: [
            {
                methods: {
                    convert_transaction: {
                        description: 'Converts an Ethereum-style transaction to Extrinsic',
                        params: [
                            {
                                name: 'transaction',
                                type: 'TransactionV2'
                            }
                        ],
                        type: 'Extrinsic'
                    }
                },
                version: 2
            },
            {
                methods: {
                    convert_transaction: {
                        description: 'Converts an Ethereum-style transaction to Extrinsic',
                        params: [
                            {
                                name: 'transaction',
                                type: 'TransactionV3'
                            }
                        ],
                        type: 'Extrinsic'
                    }
                },
                version: 3
            }
        ],
        DebugRuntimeApi: [
            {
                methods: {
                    trace_block: {
                        description: 'Trace all block extrinsics',
                        params: [
                            {
                                name: 'extrinsics',
                                type: 'Vec<Extrinsic>'
                            },
                            {
                                name: 'knownTransactions',
                                type: 'Vec<H256>'
                            }
                        ],
                        type: 'Result<(), DispatchError>'
                    },
                    trace_transaction: {
                        description: 'Trace transaction extrinsics',
                        params: [
                            {
                                name: 'extrinsics',
                                type: 'Vec<Extrinsic>'
                            },
                            {
                                name: 'transaction',
                                type: 'EthTransaction'
                            }
                        ],
                        type: 'Result<(), DispatchError>'
                    }
                },
                version: 4
            }
        ],
        EthereumRuntimeRPCApi: [
            {
                methods: {
                    ...ethMethodsV4
                },
                version: 4
            },
            {
                methods: {
                    ...ethMethodsV4,
                    ...ethMethodsV5
                },
                version: 5
            }
        ]
    };

    const V0 = {
        BlockV0: {
            header: 'EthHeader',
            transactions: 'Vec<TransactionV0>',
            ommers: 'Vec<EthHeader>'
        },
        LegacyTransaction: {
            nonce: 'U256',
            gasPrice: 'U256',
            gasLimit: 'U256',
            action: 'EthTransactionAction',
            value: 'U256',
            input: 'Bytes',
            signature: 'EthLegacyTransactionSignature'
        },
        TransactionV0: 'LegacyTransaction'
    };
    const V1 = {
        BlockV1: {
            header: 'EthHeader',
            transactions: 'Vec<TransactionV1>',
            ommers: 'Vec<EthHeader>'
        },
        EIP2930Transaction: {
            chainId: 'u64',
            nonce: 'U256',
            gasPrice: 'U256',
            gasLimit: 'U256',
            action: 'EthTransactionAction',
            value: 'U256',
            input: 'Bytes',
            accessList: 'EthAccessList',
            signature: 'EthTransactionSignature'
        },
        TransactionV1: {
            _enum: {
                Legacy: 'LegacyTransaction',
                EIP2930: 'EIP2930Transaction'
            }
        }
    };
    const V2 = {
        BlockV2: {
            header: 'EthHeader',
            transactions: 'Vec<TransactionV2>',
            ommers: 'Vec<EthHeader>'
        },
        EIP1559Transaction: {
            chainId: 'u64',
            nonce: 'U256',
            maxPriorityFeePerGas: 'U256',
            maxFeePerGas: 'U256',
            gasLimit: 'U256',
            action: 'EthTransactionAction',
            value: 'U256',
            input: 'Bytes',
            accessList: 'EthAccessList',
            signature: 'EthTransactionSignature'
        },
        TransactionV2: {
            _enum: {
                Legacy: 'LegacyTransaction',
                EIP2930: 'EIP2930Transaction',
                EIP1559: 'EIP1559Transaction'
            }
        }
    };
    const V3 = {
        BlockV3: {
            header: 'EthHeader',
            transactions: 'Vec<TransactionV3>',
            ommers: 'Vec<EthHeader>'
        },
        EIP7702Transaction: {
            chainId: 'u64',
            nonce: 'U256',
            maxPriorityFeePerGas: 'U256',
            maxFeePerGas: 'U256',
            gasLimit: 'U256',
            destination: 'EthTransactionAction',
            value: 'U256',
            data: 'Bytes',
            accessList: 'EthAccessList',
            authorizationList: 'EthAuthorizationList',
            signature: 'EthTransactionSignature'
        },
        TransactionV3: {
            _enum: {
                Legacy: 'LegacyTransaction',
                EIP2930: 'EIP2930Transaction',
                EIP1559: 'EIP1559Transaction',
                EIP7702: 'EIP7702Transaction'
            }
        }
    };
    const types = {
        ...V0,
        ...V1,
        ...V2,
        ...V3,
        EthereumAccountId: 'GenericEthereumAccountId',
        EthereumAddress: 'GenericEthereumAccountId',
        EthereumLookupSource: 'GenericEthereumLookupSource',
        EthereumSignature: '[u8; 65]',
        EthAccessListItem: {
            address: 'EthAddress',
            slots: 'Vec<H256>'
        },
        EthAccessList: 'Vec<EthAccessListItem>',
        EthAuthorizationList: 'Vec<EthAuthorizationListItem>',
        EthAuthorizationListItem: {
            chainId: 'u64',
            address: 'H160',
            nonce: 'U256',
            signature: 'EthAuthorizationSignature'
        },
        EthAuthorizationSignature: {
            oddYParity: 'bool',
            r: 'H256',
            s: 'H256'
        },
        EthAccount: {
            address: 'EthAddress',
            balance: 'U256',
            nonce: 'U256',
            codeHash: 'H256',
            storageHash: 'H256',
            accountProof: 'Vec<Bytes>',
            storageProof: 'Vec<EthStorageProof>'
        },
        EthAddress: 'H160',
        EthBlock: {
            header: 'EthHeader',
            transactions: 'Vec<EthTransaction>',
            ommers: 'Vec<EthHeader>'
        },
        EthHeader: {
            parentHash: 'H256',
            ommersHash: 'H256',
            beneficiary: 'EthAddress',
            stateRoot: 'H256',
            transactionsRoot: 'H256',
            receiptsRoot: 'H256',
            logsBloom: 'EthBloom',
            difficulty: 'U256',
            number: 'U256',
            gasLimit: 'U256',
            gasUsed: 'U256',
            timestamp: 'u64',
            extraData: 'Bytes',
            mixMash: 'H256',
            nonce: 'H64'
        },
        EthRichBlock: {
            _alias: {
                blockHash: 'hash',
                blockSize: 'size'
            },
            blockHash: 'Option<H256>',
            parentHash: 'H256',
            sha3Uncles: 'H256',
            author: 'EthAddress',
            miner: 'EthAddress',
            stateRoot: 'H256',
            transactionsRoot: 'H256',
            receiptsRoot: 'H256',
            number: 'Option<U256>',
            gasUsed: 'U256',
            gasLimit: 'U256',
            extraData: 'Bytes',
            logsBloom: 'EthBloom',
            timestamp: 'U256',
            difficulty: 'U256',
            totalDifficulty: 'Option<U256>',
            sealFields: 'Vec<Bytes>',
            uncles: 'Vec<H256>',
            transactions: 'Vec<EthTransaction>',
            blockSize: 'Option<U256>'
        },
        EthBloom: 'H2048',
        EthCallRequest: {
            from: 'Option<EthAddress>',
            to: 'Option<EthAddress>',
            gasPrice: 'Option<U256>',
            gas: 'Option<U256>',
            value: 'Option<U256>',
            data: 'Option<Bytes>',
            nonce: 'Option<U256>'
        },
        EthFeeHistory: {
            oldestBlock: 'U256',
            baseFeePerGas: 'Vec<U256>',
            gasUsedRatio: 'Vec<f64>',
            reward: 'Option<Vec<Vec<U256>>>'
        },
        EthFilter: {
            fromBlock: 'Option<BlockNumber>',
            toBlock: 'Option<BlockNumber>',
            blockHash: 'Option<H256>',
            address: 'Option<EthFilterAddress>',
            topics: 'Option<EthFilterTopic>'
        },
        EthFilterAddress: {
            _enum: {
                Single: 'EthAddress',
                Multiple: 'Vec<EthAddress>',
                Null: 'Null'
            }
        },
        EthFilterChanges: {
            _enum: {
                Logs: 'Vec<EthLog>',
                Hashes: 'Vec<H256>',
                Empty: 'Null'
            }
        },
        EthFilterTopic: {
            _enum: {
                Single: 'EthFilterTopicInner',
                Multiple: 'Vec<EthFilterTopicInner>',
                Null: 'Null'
            }
        },
        EthFilterTopicEntry: 'Option<H256>',
        EthFilterTopicInner: {
            _enum: {
                Single: 'EthFilterTopicEntry',
                Multiple: 'Vec<EthFilterTopicEntry>',
                Null: 'Null'
            }
        },
        EthRichHeader: {
            _alias: {
                blockHash: 'hash',
                blockSize: 'size'
            },
            blockHash: 'Option<H256>',
            parentHash: 'H256',
            sha3Uncles: 'H256',
            author: 'EthAddress',
            miner: 'EthAddress',
            stateRoot: 'H256',
            transactionsRoot: 'H256',
            receiptsRoot: 'H256',
            number: 'Option<U256>',
            gasUsed: 'U256',
            gasLimit: 'U256',
            extraData: 'Bytes',
            logsBloom: 'EthBloom',
            timestamp: 'U256',
            difficulty: 'U256',
            sealFields: 'Vec<Bytes>',
            blockSize: 'Option<U256>'
        },
        EthLog: {
            address: 'EthAddress',
            topics: 'Vec<H256>',
            data: 'Bytes',
            blockHash: 'Option<H256>',
            blockNumber: 'Option<U256>',
            transactionHash: 'Option<H256>',
            transactionIndex: 'Option<U256>',
            logIndex: 'Option<U256>',
            transactionLogIndex: 'Option<U256>',
            removed: 'bool'
        },
        EthReceipt: {
            transactionHash: 'Option<H256>',
            transactionIndex: 'Option<U256>',
            blockHash: 'Option<H256>',
            from: 'Option<EthAddress>',
            to: 'Option<EthAddress>',
            blockNumber: 'Option<U256>',
            cumulativeGasUsed: 'U256',
            gasUsed: 'Option<U256>',
            contractAddress: 'Option<EthAddress>',
            logs: 'Vec<EthLog>',
            root: 'Option<H256>',
            logsBloom: 'EthBloom',
            statusCode: 'Option<U64>'
        },
        EthReceiptV0: 'EthReceipt',
        EthReceiptV3: 'EthReceipt',
        EthReceiptV4: 'EthReceipt',
        EthStorageProof: {
            key: 'U256',
            value: 'U256',
            proof: 'Vec<Bytes>'
        },
        EthSubKind: {
            _enum: ['newHeads', 'logs', 'newPendingTransactions', 'syncing']
        },
        EthSubParams: {
            _enum: {
                None: 'Null',
                Logs: 'EthFilter'
            }
        },
        EthSubResult: {
            _enum: {
                Header: 'EthRichHeader',
                Log: 'EthLog',
                TransactionHash: 'H256',
                SyncState: 'EthSyncStatus'
            }
        },
        EthSyncInfo: {
            startingBlock: 'U256',
            currentBlock: 'U256',
            highestBlock: 'U256',
            warpChunksAmount: 'Option<U256>',
            warpChunksProcessed: 'Option<U256>'
        },
        EthSyncStatus: {
            _enum: {
                Info: 'EthSyncInfo',
                None: 'Null'
            }
        },
        EthTransaction: {
            hash: 'H256',
            nonce: 'U256',
            blockHash: 'Option<H256>',
            blockNumber: 'Option<U256>',
            transactionIndex: 'Option<U256>',
            from: 'H160',
            to: 'Option<H160>',
            value: 'U256',
            gasPrice: 'Option<U256>',
            maxFeePerGas: 'Option<U256>',
            maxPriorityFeePerGas: 'Option<U256>',
            gas: 'U256',
            input: 'Bytes',
            creates: 'Option<H160>',
            raw: 'Bytes',
            publicKey: 'Option<H512>',
            chainId: 'Option<U64>',
            standardV: 'U256',
            v: 'U256',
            r: 'U256',
            s: 'U256',
            accessList: 'Option<Vec<EthAccessListItem>>',
            transactionType: 'Option<U256>'
        },
        EthLegacyTransactionSignature: {
            v: 'u64',
            r: 'H256',
            s: 'H256'
        },
        EthTransactionSignature: {
            oddYParity: 'bool',
            r: 'H256',
            s: 'H256'
        },
        EthTransactionAction: {
            _enum: {
                Call: 'H160',
                Create: 'Null'
            }
        },
        EthTransactionCondition: {
            _enum: {
                block: 'u64',
                time: 'u64'
            }
        },
        EthTransactionRequest: {
            from: 'Option<EthAddress>',
            to: 'Option<EthAddress>',
            gasPrice: 'Option<U256>',
            gas: 'Option<U256>',
            value: 'Option<U256>',
            data: 'Option<Bytes>',
            nonce: 'Option<U256>'
        },
        EthTransactionStatus: {
            transactionHash: 'H256',
            transactionIndex: 'u32',
            from: 'EthAddress',
            to: 'Option<EthAddress>',
            contractAddress: 'Option<EthAddress>',
            logs: 'Vec<EthLog>',
            logsBloom: 'EthBloom'
        },
        EthWork: {
            powHash: 'H256',
            seedHash: 'H256',
            target: 'H256',
            number: 'Option<u64>'
        }
    };
    const definitions$b = { rpc: rpc$7, runtime: runtime$5, types };

    const runtime$4 = {
        AuthorFilterAPI: [
            {
                methods: {
                    can_author: {
                        description: 'The runtime api used to predict whether an author will be eligible in the given slot',
                        params: [
                            {
                                name: 'author',
                                type: 'AccountId'
                            },
                            {
                                name: 'relayParent',
                                type: 'u32'
                            },
                            {
                                name: 'parentHeader',
                                type: 'Header'
                            }
                        ],
                        type: 'bool'
                    }
                },
                version: 2
            },
            {
                methods: {
                    can_author: {
                        description: 'The runtime api used to predict whether an author will be eligible in the given slot',
                        params: [
                            {
                                name: 'author',
                                type: 'AccountId'
                            },
                            {
                                name: 'relayParent',
                                type: 'u32'
                            }
                        ],
                        type: 'bool'
                    }
                },
                version: 1
            }
        ],
        NimbusApi: [
            {
                methods: {
                    can_author: {
                        description: 'The runtime api used to predict whether a Nimbus author will be eligible in the given slot',
                        params: [
                            {
                                name: 'author',
                                type: 'AccountId'
                            },
                            {
                                name: 'relayParent',
                                type: 'u32'
                            },
                            {
                                name: 'parentHeader',
                                type: 'Header'
                            }
                        ],
                        type: 'bool'
                    }
                },
                version: 1
            }
        ]
    };

    const definitions$a = {
        rpc: {},
        runtime: runtime$4,
        types: {}
    };

    const runtime$3 = {
        OracleApi: [
            {
                methods: {
                    get_all_values: {
                        description: 'Retrieves all values',
                        params: [
                            {
                                name: 'providerId',
                                type: 'Raw'
                            }
                        ],
                        type: 'Raw'
                    },
                    get_value: {
                        description: 'Retrieves a single value',
                        params: [
                            {
                                name: 'providerId',
                                type: 'Raw'
                            },
                            {
                                name: 'key',
                                type: 'Raw'
                            }
                        ],
                        type: 'Option<Raw>'
                    }
                },
                version: 1
            }
        ]
    };

    const definitions$9 = {
        rpc: {},
        runtime: runtime$3,
        types: {}
    };

    const runtime$2 = {
        TokensApi: [
            {
                methods: {
                    query_existential_deposit: {
                        description: 'Query the existential amount for a specific currency',
                        params: [
                            {
                                name: 'currencyId',
                                type: 'Raw'
                            }
                        ],
                        type: 'u128'
                    }
                },
                version: 1
            }
        ]
    };

    const definitions$8 = {
        rpc: {},
        runtime: runtime$2,
        types: {}
    };

    const rpc$6 = {
        methods: {
            description: 'Retrieves the list of RPC methods that are exposed by the node',
            params: [],
            type: 'RpcMethods'
        }
    };

    const definitions$7 = {
        rpc: rpc$6,
        types: {
            RpcMethods: {
                version: 'u32',
                methods: 'Vec<Text>'
            }
        }
    };

    const rpc$5 = {
        hasKey: {
            description: 'Returns true if the keystore has private keys for the given public key and key type.',
            isUnsafe: true,
            params: [
                {
                    name: 'publicKey',
                    type: 'Bytes'
                },
                {
                    name: 'keyType',
                    type: 'Text'
                }
            ],
            type: 'bool'
        },
        hasSessionKeys: {
            description: 'Returns true if the keystore has private keys for the given session public keys.',
            isUnsafe: true,
            params: [
                {
                    name: 'sessionKeys',
                    type: 'Bytes'
                }
            ],
            type: 'bool'
        },
        insertKey: {
            description: 'Insert a key into the keystore.',
            isUnsafe: true,
            params: [
                {
                    name: 'keyType',
                    type: 'Text'
                },
                {
                    name: 'suri',
                    type: 'Text'
                },
                {
                    name: 'publicKey',
                    type: 'Bytes'
                }
            ],
            type: 'Bytes'
        },
        pendingExtrinsics: {
            description: 'Returns all pending extrinsics, potentially grouped by sender',
            params: [],
            type: 'Vec<Extrinsic>'
        },
        removeExtrinsic: {
            description: 'Remove given extrinsic from the pool and temporarily ban it to prevent reimporting',
            isUnsafe: true,
            params: [
                {
                    name: 'bytesOrHash',
                    type: 'Vec<ExtrinsicOrHash>'
                }
            ],
            type: 'Vec<Hash>'
        },
        rotateKeys: {
            description: 'Generate new session keys and returns the corresponding public keys',
            isUnsafe: true,
            params: [],
            type: 'Bytes'
        },
        submitAndWatchExtrinsic: {
            description: 'Submit and subscribe to watch an extrinsic until unsubscribed',
            isSigned: true,
            params: [
                {
                    name: 'extrinsic',
                    type: 'Extrinsic'
                }
            ],
            pubsub: [
                'extrinsicUpdate',
                'submitAndWatchExtrinsic',
                'unwatchExtrinsic'
            ],
            type: 'ExtrinsicStatus'
        },
        submitExtrinsic: {
            description: 'Submit a fully formatted extrinsic for block inclusion',
            isSigned: true,
            params: [
                {
                    name: 'extrinsic',
                    type: 'Extrinsic'
                }
            ],
            type: 'Hash'
        }
    };

    const definitions$6 = {
        rpc: rpc$5,
        types: {
            ExtrinsicOrHash: {
                _enum: {
                    Hash: 'Hash',
                    Extrinsic: 'Bytes'
                }
            },
            ExtrinsicStatus: {
                _enum: {
                    Future: 'Null',
                    Ready: 'Null',
                    Broadcast: 'Vec<Text>',
                    InBlock: 'Hash',
                    Retracted: 'Hash',
                    FinalityTimeout: 'Hash',
                    Finalized: 'Hash',
                    Usurped: 'Hash',
                    Dropped: 'Null',
                    Invalid: 'Null'
                }
            }
        }
    };

    const rpc$4 = {
        getBlock: {
            description: 'Get header and body of a relay chain block',
            params: [
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'hash',
                    type: 'BlockHash'
                }
            ],
            type: 'SignedBlock'
        },
        getBlockHash: {
            description: 'Get the block hash for a specific block',
            params: [
                {
                    isOptional: true,
                    name: 'blockNumber',
                    type: 'BlockNumber'
                }
            ],
            type: 'BlockHash'
        },
        getFinalizedHead: {
            alias: ['chain_getFinalisedHead'],
            description: 'Get hash of the last finalized block in the canon chain',
            params: [],
            type: 'BlockHash'
        },
        getHeader: {
            alias: ['chain_getHead'],
            description: 'Retrieves the header for a specific block',
            params: [
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'hash',
                    type: 'BlockHash'
                }
            ],
            type: 'Header'
        },
        subscribeAllHeads: {
            description: 'Retrieves the newest header via subscription',
            params: [],
            pubsub: [
                'allHead',
                'subscribeAllHeads',
                'unsubscribeAllHeads'
            ],
            type: 'Header'
        },
        subscribeFinalizedHeads: {
            alias: ['chain_subscribeFinalisedHeads', 'chain_unsubscribeFinalisedHeads'],
            description: 'Retrieves the best finalized header via subscription',
            params: [],
            pubsub: [
                'finalizedHead',
                'subscribeFinalizedHeads',
                'unsubscribeFinalizedHeads'
            ],
            type: 'Header'
        },
        subscribeNewHeads: {
            alias: ['chain_unsubscribeNewHeads', 'subscribe_newHead', 'unsubscribe_newHead'],
            description: 'Retrieves the best header via subscription',
            params: [],
            pubsub: [
                'newHead',
                'subscribeNewHead',
                'unsubscribeNewHead'
            ],
            type: 'Header'
        }
    };

    const definitions$5 = {
        rpc: rpc$4,
        types: {
            BlockHash: 'Hash'
        }
    };

    const rpc$3 = {
        getKeys: {
            description: 'Returns the keys with prefix from a child storage, leave empty to get all the keys',
            params: [
                {
                    name: 'childKey',
                    type: 'PrefixedStorageKey'
                },
                {
                    name: 'prefix',
                    type: 'StorageKey'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'Hash'
                }
            ],
            type: 'Vec<StorageKey>'
        },
        getKeysPaged: {
            alias: ['childstate_getKeysPagedAt'],
            description: 'Returns the keys with prefix from a child storage with pagination support',
            params: [
                {
                    name: 'childKey',
                    type: 'PrefixedStorageKey'
                },
                {
                    name: 'prefix',
                    type: 'StorageKey'
                },
                {
                    name: 'count',
                    type: 'u32'
                },
                {
                    isOptional: true,
                    name: 'startKey',
                    type: 'StorageKey'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'Hash'
                }
            ],
            type: 'Vec<StorageKey>'
        },
        getStorage: {
            description: 'Returns a child storage entry at a specific block state',
            params: [
                {
                    name: 'childKey',
                    type: 'PrefixedStorageKey'
                },
                {
                    name: 'key',
                    type: 'StorageKey'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'Hash'
                }
            ],
            type: 'Option<StorageData>'
        },
        getStorageEntries: {
            description: 'Returns child storage entries for multiple keys at a specific block state',
            params: [
                {
                    name: 'childKey',
                    type: 'PrefixedStorageKey'
                },
                {
                    name: 'keys',
                    type: 'Vec<StorageKey>'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'Hash'
                }
            ],
            type: 'Vec<Option<StorageData>>'
        },
        getStorageHash: {
            description: 'Returns the hash of a child storage entry at a block state',
            params: [
                {
                    name: 'childKey',
                    type: 'PrefixedStorageKey'
                },
                {
                    name: 'key',
                    type: 'StorageKey'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'Hash'
                }
            ],
            type: 'Option<Hash>'
        },
        getStorageSize: {
            description: 'Returns the size of a child storage entry at a block state',
            params: [
                {
                    name: 'childKey',
                    type: 'PrefixedStorageKey'
                },
                {
                    name: 'key',
                    type: 'StorageKey'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'Hash'
                }
            ],
            type: 'Option<u64>'
        }
    };

    const definitions$4 = {
        rpc: rpc$3,
        types: {
            PrefixedStorageKey: 'StorageKey'
        }
    };

    const rpc$2 = {
        localStorageClear: {
            description: 'Clear offchain local storage under given key and prefix',
            isUnsafe: true,
            params: [
                {
                    name: 'kind',
                    type: 'StorageKind'
                },
                {
                    name: 'key',
                    type: 'Bytes'
                }
            ],
            type: 'Null'
        },
        localStorageGet: {
            description: 'Get offchain local storage under given key and prefix',
            isUnsafe: true,
            params: [
                {
                    name: 'kind',
                    type: 'StorageKind'
                },
                {
                    name: 'key',
                    type: 'Bytes'
                }
            ],
            type: 'Option<Bytes>'
        },
        localStorageSet: {
            description: 'Set offchain local storage under given key and prefix',
            isUnsafe: true,
            params: [
                {
                    name: 'kind',
                    type: 'StorageKind'
                },
                {
                    name: 'key',
                    type: 'Bytes'
                },
                {
                    name: 'value',
                    type: 'Bytes'
                }
            ],
            type: 'Null'
        }
    };

    const runtime$1 = {
        OffchainWorkerApi: [
            {
                methods: {
                    offchain_worker: {
                        description: 'Starts the off-chain task for given block header.',
                        params: [
                            {
                                name: 'header',
                                type: 'Header'
                            }
                        ],
                        type: 'Null'
                    }
                },
                version: 2
            },
            {
                methods: {
                    offchain_worker: {
                        description: 'Starts the off-chain task for given block header.',
                        params: [
                            {
                                name: 'number',
                                type: 'BlockNumber'
                            }
                        ],
                        type: 'Null'
                    }
                },
                version: 1
            }
        ]
    };

    const definitions$3 = {
        rpc: rpc$2,
        runtime: runtime$1,
        types: {
            StorageKind: {
                _enum: {
                    PERSISTENT: 1,
                    LOCAL: 2
                }
            }
        }
    };

    const rpc$1 = {
        queryFeeDetails: {
            deprecated: 'Use `api.call.transactionPaymentApi.queryFeeDetails` instead',
            description: 'Query the detailed fee of a given encoded extrinsic',
            params: [
                {
                    name: 'extrinsic',
                    type: 'Bytes'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'FeeDetails'
        },
        queryInfo: {
            deprecated: 'Use `api.call.transactionPaymentApi.queryInfo` instead',
            description: 'Retrieves the fee information for an encoded extrinsic',
            params: [
                {
                    name: 'extrinsic',
                    type: 'Bytes'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'RuntimeDispatchInfoV1'
        }
    };

    const V1_TO_V4_SHARED_PAY = {
        query_fee_details: {
            description: 'The transaction fee details',
            params: [
                {
                    name: 'uxt',
                    type: 'Extrinsic'
                },
                {
                    name: 'len',
                    type: 'u32'
                }
            ],
            type: 'FeeDetails'
        }
    };
    const V1_TO_V3_SHARED_CALL = {
        query_call_fee_details: {
            description: 'The call fee details',
            params: [
                {
                    name: 'call',
                    type: 'Call'
                },
                {
                    name: 'len',
                    type: 'u32'
                }
            ],
            type: 'FeeDetails'
        }
    };
    const V2_TO_V4_SHARED_PAY = {
        query_info: {
            description: 'The transaction info',
            params: [
                {
                    name: 'uxt',
                    type: 'Extrinsic'
                },
                {
                    name: 'len',
                    type: 'u32'
                }
            ],
            type: 'RuntimeDispatchInfo'
        }
    };
    const V2_V3_SHARED_CALL = {
        query_call_info: {
            description: 'The call info',
            params: [
                {
                    name: 'call',
                    type: 'Call'
                },
                {
                    name: 'len',
                    type: 'u32'
                }
            ],
            type: 'RuntimeDispatchInfo'
        }
    };
    const V3_SHARED_PAY_CALL = {
        query_length_to_fee: {
            description: 'Query the output of the current LengthToFee given some input',
            params: [
                {
                    name: 'length',
                    type: 'u32'
                }
            ],
            type: 'Balance'
        },
        query_weight_to_fee: {
            description: 'Query the output of the current WeightToFee given some input',
            params: [
                {
                    name: 'weight',
                    type: 'Weight'
                }
            ],
            type: 'Balance'
        }
    };
    const runtime = {
        TransactionPaymentApi: [
            {
                methods: {
                    ...V3_SHARED_PAY_CALL,
                    ...V2_TO_V4_SHARED_PAY,
                    ...V1_TO_V4_SHARED_PAY
                },
                version: 4
            },
            {
                methods: {
                    ...V3_SHARED_PAY_CALL,
                    ...V2_TO_V4_SHARED_PAY,
                    ...V1_TO_V4_SHARED_PAY
                },
                version: 3
            },
            {
                methods: {
                    ...V2_TO_V4_SHARED_PAY,
                    ...V1_TO_V4_SHARED_PAY
                },
                version: 2
            },
            {
                methods: {
                    query_info: {
                        description: 'The transaction info',
                        params: [
                            {
                                name: 'uxt',
                                type: 'Extrinsic'
                            },
                            {
                                name: 'len',
                                type: 'u32'
                            }
                        ],
                        type: 'RuntimeDispatchInfo'
                    },
                    ...V1_TO_V4_SHARED_PAY
                },
                version: 1
            }
        ],
        TransactionPaymentCallApi: [
            {
                methods: {
                    ...V3_SHARED_PAY_CALL,
                    ...V2_V3_SHARED_CALL,
                    ...V1_TO_V3_SHARED_CALL
                },
                version: 3
            },
            {
                methods: {
                    ...V2_V3_SHARED_CALL,
                    ...V1_TO_V3_SHARED_CALL
                },
                version: 2
            },
            {
                methods: {
                    CALL: {
                        description: 'The call info',
                        params: [
                            {
                                name: 'call',
                                type: 'Call'
                            },
                            {
                                name: 'len',
                                type: 'u32'
                            }
                        ],
                        type: 'RuntimeDispatchInfo'
                    },
                    ...V1_TO_V3_SHARED_CALL
                },
                version: 1
            }
        ]
    };

    const definitions$2 = {
        rpc: rpc$1,
        runtime,
        types: {
            FeeDetails: {
                inclusionFee: 'Option<InclusionFee>'
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

    const rpc = {
        call: {
            alias: ['state_callAt'],
            description: 'Perform a call to a builtin on the chain',
            params: [
                {
                    name: 'method',
                    type: 'Text'
                },
                {
                    name: 'data',
                    type: 'Bytes'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'Bytes'
        },
        getChildKeys: {
            description: 'Retrieves the keys with prefix of a specific child storage',
            params: [
                {
                    name: 'childStorageKey',
                    type: 'StorageKey'
                },
                {
                    name: 'childDefinition',
                    type: 'StorageKey'
                },
                {
                    name: 'childType',
                    type: 'u32'
                },
                {
                    name: 'key',
                    type: 'StorageKey'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'Vec<StorageKey>'
        },
        getChildReadProof: {
            description: 'Returns proof of storage for child key entries at a specific block state.',
            params: [
                {
                    name: 'childStorageKey',
                    type: 'PrefixedStorageKey'
                },
                {
                    name: 'keys',
                    type: 'Vec<StorageKey>'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'ReadProof'
        },
        getChildStorage: {
            description: 'Retrieves the child storage for a key',
            params: [
                {
                    name: 'childStorageKey',
                    type: 'StorageKey'
                },
                {
                    name: 'childDefinition',
                    type: 'StorageKey'
                },
                {
                    name: 'childType',
                    type: 'u32'
                },
                {
                    name: 'key',
                    type: 'StorageKey'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'StorageData'
        },
        getChildStorageHash: {
            description: 'Retrieves the child storage hash',
            params: [
                {
                    name: 'childStorageKey',
                    type: 'StorageKey'
                },
                {
                    name: 'childDefinition',
                    type: 'StorageKey'
                },
                {
                    name: 'childType',
                    type: 'u32'
                },
                {
                    name: 'key',
                    type: 'StorageKey'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'Hash'
        },
        getChildStorageSize: {
            description: 'Retrieves the child storage size',
            params: [
                {
                    name: 'childStorageKey',
                    type: 'StorageKey'
                },
                {
                    name: 'childDefinition',
                    type: 'StorageKey'
                },
                {
                    name: 'childType',
                    type: 'u32'
                },
                {
                    name: 'key',
                    type: 'StorageKey'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'u64'
        },
        getKeys: {
            deprecated: 'Use `api.rpc.state.getKeysPaged` to retrieve keys',
            description: 'Retrieves the keys with a certain prefix',
            params: [
                {
                    name: 'key',
                    type: 'StorageKey'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'Vec<StorageKey>'
        },
        getKeysPaged: {
            alias: ['state_getKeysPagedAt'],
            description: 'Returns the keys with prefix with pagination support.',
            params: [
                {
                    name: 'key',
                    type: 'StorageKey'
                },
                {
                    name: 'count',
                    type: 'u32'
                },
                {
                    isOptional: true,
                    name: 'startKey',
                    type: 'StorageKey'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'Vec<StorageKey>'
        },
        getMetadata: {
            description: 'Returns the runtime metadata',
            params: [
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'Metadata'
        },
        getPairs: {
            deprecated: 'Use `api.rpc.state.getKeysPaged` to retrieve keys',
            description: 'Returns the keys with prefix, leave empty to get all the keys (deprecated: Use getKeysPaged)',
            isUnsafe: true,
            params: [
                {
                    name: 'prefix',
                    type: 'StorageKey'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'Vec<KeyValue>'
        },
        getReadProof: {
            description: 'Returns proof of storage entries at a specific block state',
            params: [
                {
                    name: 'keys',
                    type: 'Vec<StorageKey>'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'ReadProof'
        },
        getRuntimeVersion: {
            alias: ['chain_getRuntimeVersion'],
            description: 'Get the runtime version',
            params: [
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'RuntimeVersion'
        },
        getStorage: {
            alias: ['state_getStorageAt'],
            description: 'Retrieves the storage for a key',
            params: [
                {
                    name: 'key',
                    type: 'StorageKey'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'StorageData'
        },
        getStorageHash: {
            alias: ['state_getStorageHashAt'],
            description: 'Retrieves the storage hash',
            params: [
                {
                    name: 'key',
                    type: 'StorageKey'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'Hash'
        },
        getStorageSize: {
            alias: ['state_getStorageSizeAt'],
            description: 'Retrieves the storage size',
            params: [
                {
                    name: 'key',
                    type: 'StorageKey'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'u64'
        },
        queryStorage: {
            description: 'Query historical storage entries (by key) starting from a start block',
            isUnsafe: true,
            params: [
                {
                    name: 'keys',
                    type: 'Vec<StorageKey>'
                },
                {
                    name: 'fromBlock',
                    type: 'Hash'
                },
                {
                    isOptional: true,
                    name: 'toBlock',
                    type: 'BlockHash'
                }
            ],
            type: 'Vec<StorageChangeSet>'
        },
        queryStorageAt: {
            description: 'Query storage entries (by key) starting at block hash given as the second parameter',
            params: [
                {
                    name: 'keys',
                    type: 'Vec<StorageKey>'
                },
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'Vec<StorageChangeSet>'
        },
        subscribeRuntimeVersion: {
            alias: ['chain_subscribeRuntimeVersion', 'chain_unsubscribeRuntimeVersion'],
            description: 'Retrieves the runtime version via subscription',
            params: [],
            pubsub: [
                'runtimeVersion',
                'subscribeRuntimeVersion',
                'unsubscribeRuntimeVersion'
            ],
            type: 'RuntimeVersion'
        },
        subscribeStorage: {
            description: 'Subscribes to storage changes for the provided keys',
            params: [
                {
                    isOptional: true,
                    name: 'keys',
                    type: 'Vec<StorageKey>'
                }
            ],
            pubsub: [
                'storage',
                'subscribeStorage',
                'unsubscribeStorage'
            ],
            type: 'StorageChangeSet'
        },
        traceBlock: {
            description: 'Provides a way to trace the re-execution of a single block',
            isUnsafe: true,
            params: [
                {
                    name: 'block',
                    type: 'Hash'
                },
                {
                    name: 'targets',
                    type: 'Option<Text>'
                },
                {
                    name: 'storageKeys',
                    type: 'Option<Text>'
                },
                {
                    name: 'methods',
                    type: 'Option<Text>'
                }
            ],
            type: 'TraceBlockResponse'
        },
        trieMigrationStatus: {
            description: 'Check current migration state',
            isUnsafe: true,
            params: [
                {
                    isHistoric: true,
                    isOptional: true,
                    name: 'at',
                    type: 'BlockHash'
                }
            ],
            type: 'MigrationStatusResult'
        }
    };

    const definitions$1 = {
        rpc,
        types: {
            ApiId: '[u8; 8]',
            BlockTrace: {
                blockHash: 'Text',
                parentHash: 'Text',
                tracingTargets: 'Text',
                storageKeys: 'Text',
                spans: 'Vec<BlockTraceSpan>',
                events: 'Vec<BlockTraceEvent>'
            },
            BlockTraceEvent: {
                target: 'Text',
                data: 'BlockTraceEventData',
                parentId: 'Option<u64>'
            },
            BlockTraceEventData: {
                stringValues: 'HashMap<Text, Text>'
            },
            BlockTraceSpan: {
                id: 'u64',
                parentId: 'Option<u64>',
                name: 'Text',
                target: 'Text',
                wasm: 'bool'
            },
            KeyValueOption: '(StorageKey, Option<StorageData>)',
            MigrationStatusResult: {
                topRemainingToMigrate: 'u64',
                childRemainingToMigrate: 'u64'
            },
            ReadProof: {
                at: 'Hash',
                proof: 'Vec<Bytes>'
            },
            RuntimeVersionApi: '(ApiId, u32)',
            RuntimeVersion: {
                specName: 'Text',
                implName: 'Text',
                authoringVersion: 'u32',
                specVersion: 'u32',
                implVersion: 'u32',
                apis: 'Vec<RuntimeVersionApi>',
                transactionVersion: 'u32',
                stateVersion: 'u8'
            },
            RuntimeVersionPre4: {
                specName: 'Text',
                implName: 'Text',
                authoringVersion: 'u32',
                specVersion: 'u32',
                implVersion: 'u32',
                apis: 'Vec<RuntimeVersionApi>',
                transactionVersion: 'u32'
            },
            RuntimeVersionPre3: {
                specName: 'Text',
                implName: 'Text',
                authoringVersion: 'u32',
                specVersion: 'u32',
                implVersion: 'u32',
                apis: 'Vec<RuntimeVersionApi>'
            },
            RuntimeVersionPartial: {
                specName: 'Text',
                specVersion: 'u32',
                apis: 'Vec<RuntimeVersionApi>'
            },
            SpecVersion: 'u32',
            StorageChangeSet: {
                block: 'Hash',
                changes: 'Vec<KeyValueOption>'
            },
            TraceBlockResponse: {
                _enum: {
                    TraceError: 'TraceError',
                    BlockTrace: 'BlockTrace'
                }
            },
            TraceError: {
                error: 'Text'
            }
        }
    };

    const definitions = /*#__PURE__*/Object.freeze({
        __proto__: null,
        assetConversion: definitions$1a,
        assets: definitions$19,
        attestations: definitions$p,
        aura: definitions$18,
        author: definitions$6,
        authorship: definitions$17,
        babe: definitions$16,
        balances: definitions$15,
        beefy: definitions$14,
        benchmark: definitions$13,
        blockbuilder: definitions$12,
        bridges: definitions$o,
        chain: definitions$5,
        childstate: definitions$4,
        claims: definitions$n,
        collective: definitions$11,
        consensus: definitions$10,
        contracts: definitions$$,
        contractsAbi: definitions$c,
        crowdloan: definitions$m,
        cumulus: definitions$l,
        democracy: definitions$_,
        dev: definitions$Z,
        discovery: definitions$Y,
        dryRunApi: definitions$g,
        elections: definitions$X,
        engine: definitions$W,
        eth: definitions$b,
        evm: definitions$V,
        extrinsics: definitions$U,
        finality: definitions$k,
        fungibles: definitions$T,
        genericAsset: definitions$S,
        genesisBuilder: definitions$R,
        gilt: definitions$Q,
        grandpa: definitions$P,
        identity: definitions$O,
        imOnline: definitions$N,
        lottery: definitions$M,
        metadata: definitions$1d,
        mixnet: definitions$L,
        mmr: definitions$K,
        nfts: definitions$J,
        nimbus: definitions$a,
        nompools: definitions$I,
        offchain: definitions$3,
        offences: definitions$H,
        ormlOracle: definitions$9,
        ormlTokens: definitions$8,
        parachains: definitions$j,
        payment: definitions$2,
        poll: definitions$i,
        pow: definitions$G,
        proxy: definitions$F,
        purchase: definitions$h,
        recovery: definitions$E,
        rpc: definitions$7,
        runtime: definitions$1c,
        scaleInfo: definitions$1b,
        scheduler: definitions$D,
        session: definitions$C,
        society: definitions$B,
        staking: definitions$A,
        state: definitions$1,
        statement: definitions$z,
        support: definitions$y,
        syncstate: definitions$x,
        system: definitions$w,
        treasury: definitions$v,
        txpayment: definitions$u,
        txqueue: definitions$t,
        uniques: definitions$s,
        utility: definitions$r,
        vesting: definitions$q,
        xcm: definitions$f,
        xcmPaymentApi: definitions$e,
        xcmRuntimeApi: definitions$d
    });

    const jsonrpc = {};
    Object.keys(definitions).forEach((s) => Object.entries(definitions[s].rpc || {}).forEach(([method, def]) => {
        const section = def.aliasSection || s;
        if (!jsonrpc[section]) {
            jsonrpc[section] = {};
        }
        jsonrpc[section][method] = util.objectSpread({}, def, {
            isSubscription: !!def.pubsub,
            jsonrpc: `${section}_${method}`,
            method,
            section
        });
    }));
    const jsonrpc$1 = jsonrpc;

    const l$2 = util.logger('PortableRegistry');
    const TYPE_UNWRAP = { toNumber: () => -1 };
    const PRIMITIVE_ALIAS = {
        Char: 'u32',
        Str: 'Text'
    };
    const PATHS_ALIAS = splitNamespace([
        'sp_core::crypto::AccountId32',
        'sp_runtime::generic::era::Era',
        'sp_runtime::multiaddress::MultiAddress',
        'fp_account::AccountId20',
        'account::AccountId20',
        'polkadot_runtime_common::claims::EthereumAddress',
        'frame_support::weights::weight_v2::Weight',
        'sp_weights::weight_v2::Weight',
        '*_democracy::vote::Vote',
        '*_conviction_voting::vote::Vote',
        '*_identity::types::Data',
        'sp_core::OpaqueMetadata',
        'sp_core::OpaquePeerId',
        'sp_core::offchain::OpaqueMultiaddr',
        'primitive_types::*',
        'sp_arithmetic::per_things::*',
        '*_runtime::RuntimeCall',
        '*_runtime::RuntimeEvent',
        'ink::env::types::*',
        'ink::primitives::types::*',
        'ink_env::types::*',
        'ink_primitives::types::*'
    ]);
    const PATHS_SET = splitNamespace([
        'pallet_identity::types::BitFlags'
    ]);
    const BITVEC_NS_LSB = ['bitvec::order::Lsb0', 'BitOrderLsb0'];
    const BITVEC_NS_MSB = ['bitvec::order::Msb0', 'BitOrderMsb0'];
    const BITVEC_NS = [...BITVEC_NS_LSB, ...BITVEC_NS_MSB];
    const WRAPPERS = ['BoundedBTreeMap', 'BoundedBTreeSet', 'BoundedVec', 'Box', 'BTreeMap', 'BTreeSet', 'Cow', 'Option', 'Range', 'RangeInclusive', 'Result', 'WeakBoundedVec', 'WrapperKeepOpaque', 'WrapperOpaque'];
    const RESERVED = [
        'entries', 'keys', 'new', 'size',
        'hash', 'registry'
    ];
    const PATH_RM_INDEX_1 = ['generic', 'misc', 'pallet', 'traits', 'types'];
    function sanitizeDocs(docs) {
        const count = docs.length;
        const result = new Array(count);
        for (let i = 0; i < count; i++) {
            result[i] = docs[i].toString();
        }
        return result;
    }
    function splitNamespace(values) {
        const count = values.length;
        const result = new Array(count);
        for (let i = 0; i < count; i++) {
            result[i] = values[i].split('::');
        }
        return result;
    }
    function matchParts(first, second) {
        return first.length === second.length && first.every((a, index) => {
            const b = second[index].toString();
            if ((a === '*') || (a === b)) {
                return true;
            }
            if (a.includes('*') && a.includes('_') && b.includes('_')) {
                let suba = a.split('_');
                let subb = b.split('_');
                if (suba[0] === '*') {
                    const indexOf = subb.indexOf(suba[1]);
                    if (indexOf !== -1) {
                        suba = suba.slice(1);
                        subb = subb.slice(indexOf);
                    }
                }
                if ((suba.length === 2) && (suba[1] === '*') && (suba[0] === subb[0])) {
                    return true;
                }
                return matchParts(suba, subb);
            }
            return false;
        });
    }
    function getAliasPath({ def, path }) {
        if (['frame_support::weights::weight_v2::Weight', 'sp_weights::weight_v2::Weight'].includes(path.join('::'))) {
            return !def.isComposite || def.asComposite.fields.length === 1
                ? 'WeightV1'
                : null;
        }
        return path.length && PATHS_ALIAS.some((a) => matchParts(a, path))
            ? path[path.length - 1].toString()
            : null;
    }
    function extractNameFlat(portable, lookupIndex, params, path, isInternal = false) {
        const count = path.length;
        if (count === 0 || WRAPPERS.includes(path[count - 1].toString())) {
            return null;
        }
        const camels = new Array(count);
        const lowers = new Array(count);
        for (let i = 0; i < count; i++) {
            const c = util.stringPascalCase(isInternal
                ? path[i].replace('pallet_', '')
                : path[i]);
            const l = c.toLowerCase();
            camels[i] = c;
            lowers[i] = l;
        }
        let name = '';
        for (let i = 0; i < count; i++) {
            const l = lowers[i];
            if (i !== 1 || !PATH_RM_INDEX_1.includes(l)) {
                if (l !== lowers[i + 1]) {
                    name += camels[i];
                }
            }
        }
        if (camels[1] === 'RawOrigin' && count === 2 && params.length === 2 && params[1].type.isSome) {
            const instanceType = portable[params[1].type.unwrap().toNumber()];
            if (instanceType.type.path.length === 2) {
                name = `${name}${instanceType.type.path[1].toString()}`;
            }
        }
        return { lookupIndex, name, params };
    }
    function extractName(portable, lookupIndex, { type: { params, path } }) {
        return extractNameFlat(portable, lookupIndex, params, path);
    }
    function nextDupeMatches(name, startAt, names) {
        const result = [names[startAt]];
        for (let i = startAt + 1, count = names.length; i < count; i++) {
            const v = names[i];
            if (v.name === name) {
                result.push(v);
            }
        }
        return result;
    }
    function rewriteDupes(input, rewrite) {
        const count = input.length;
        for (let i = 0; i < count; i++) {
            const a = input[i];
            for (let j = i + 1; j < count; j++) {
                const b = input[j];
                if (a.lookupIndex !== b.lookupIndex && a.name === b.name) {
                    return false;
                }
            }
        }
        for (let i = 0; i < count; i++) {
            const p = input[i];
            rewrite[p.lookupIndex] = p.name;
        }
        return true;
    }
    function removeDupeNames(lookup, portable, names) {
        const rewrite = {};
        return names
            .map((original, startAt) => {
            const { lookupIndex, name, params } = original;
            if (!name) {
                return null;
            }
            else if (rewrite[lookupIndex]) {
                return original;
            }
            const allSame = nextDupeMatches(name, startAt, names);
            if (allSame.length === 1) {
                return original;
            }
            const anyDiff = allSame.some((o) => params.length !== o.params.length ||
                params.some((p, index) => !p.name.eq(o.params[index].name) ||
                    p.type.unwrapOr(TYPE_UNWRAP).toNumber() !== o.params[index].type.unwrapOr(TYPE_UNWRAP).toNumber()));
            if (!anyDiff) {
                return original;
            }
            const paramIdx = params.findIndex(({ type }, index) => allSame.every(({ params }, aIndex) => params[index].type.isSome && (aIndex === 0 ||
                !params[index].type.eq(type))));
            if (paramIdx === -1) {
                return original;
            }
            const sameCount = allSame.length;
            const adjusted = new Array(sameCount);
            for (let i = 0; i < sameCount; i++) {
                const { lookupIndex, name, params } = allSame[i];
                const { def, path } = lookup.getSiType(params[paramIdx].type.unwrap());
                if (!def.isPrimitive && !path.length) {
                    return null;
                }
                adjusted[i] = {
                    lookupIndex,
                    name: def.isPrimitive
                        ? `${name}${def.asPrimitive.toString()}`
                        : `${name}${path[path.length - 1].toString()}`
                };
            }
            if (rewriteDupes(adjusted, rewrite)) {
                return original;
            }
            for (let i = 0; i < sameCount; i++) {
                const { lookupIndex, name, params } = allSame[i];
                const { def, path } = lookup.getSiType(params[paramIdx].type.unwrap());
                const flat = extractNameFlat(portable, lookupIndex, params, path, true);
                if (def.isPrimitive || !flat) {
                    return null;
                }
                adjusted[i] = {
                    lookupIndex,
                    name: `${name}${flat.name}`
                };
            }
            if (rewriteDupes(adjusted, rewrite)) {
                return original;
            }
            return null;
        })
            .filter((n) => !!n)
            .map(({ lookupIndex, name, params }) => ({
            lookupIndex,
            name: rewrite[lookupIndex] || name,
            params
        }));
    }
    function registerTypes(lookup, lookups, names, params) {
        lookup.registry.register(lookups);
        if (params.SpRuntimeUncheckedExtrinsic) {
            const [addrParam, , sigParam] = params.SpRuntimeUncheckedExtrinsic;
            const siAddress = lookup.getSiType(addrParam.type.unwrap());
            const siSignature = lookup.getSiType(sigParam.type.unwrap());
            const nsSignature = siSignature.path.join('::');
            let nsAccountId = siAddress.path.join('::');
            const isMultiAddress = nsAccountId === 'sp_runtime::multiaddress::MultiAddress';
            if (isMultiAddress) {
                const [idParam] = siAddress.params;
                nsAccountId = lookup.getSiType(idParam.type.unwrap()).path.join('::');
            }
            lookup.registry.register({
                AccountId: nsAccountId.endsWith('::AccountId20') || nsAccountId.endsWith('::H160')
                    ? 'AccountId20'
                    : 'AccountId32',
                Address: isMultiAddress
                    ? 'MultiAddress'
                    : 'AccountId',
                ExtrinsicSignature: ['sp_runtime::MultiSignature'].includes(nsSignature)
                    ? 'MultiSignature'
                    : names[sigParam.type.unwrap().toNumber()] || 'MultiSignature'
            });
        }
    }
    function extractAliases(params, isContract) {
        const hasParams = Object.keys(params).some((k) => !k.startsWith('Pallet'));
        const alias = {};
        if (params.SpRuntimeUncheckedExtrinsic) {
            const [, { type }] = params.SpRuntimeUncheckedExtrinsic;
            alias[type.unwrap().toNumber()] = 'Call';
        }
        else if (hasParams && !isContract) {
            l$2.warn('Unable to determine runtime Call type, cannot inspect sp_runtime::generic::unchecked_extrinsic::UncheckedExtrinsic');
        }
        if (params.FrameSystemEventRecord) {
            const [{ type }] = params.FrameSystemEventRecord;
            alias[type.unwrap().toNumber()] = 'Event';
        }
        else if (hasParams && !isContract) {
            l$2.warn('Unable to determine runtime Event type, cannot inspect frame_system::EventRecord');
        }
        return alias;
    }
    function extractTypeInfo(lookup, portable) {
        const nameInfo = [];
        const types = {};
        for (let i = 0, count = portable.length; i < count; i++) {
            const type = portable[i];
            const lookupIndex = type.id.toNumber();
            const extracted = extractName(portable, lookupIndex, portable[i]);
            if (extracted) {
                nameInfo.push(extracted);
            }
            types[lookupIndex] = type;
        }
        const lookups = {};
        const names = {};
        const params = {};
        const dedup = removeDupeNames(lookup, portable, nameInfo);
        for (let i = 0, count = dedup.length; i < count; i++) {
            const { lookupIndex, name, params: p } = dedup[i];
            names[lookupIndex] = name;
            lookups[name] = lookup.registry.createLookupType(lookupIndex);
            params[name] = p;
        }
        return { lookups, names, params, types };
    }
    class PortableRegistry extends Struct {
        #alias;
        #lookups;
        #names;
        #params;
        #typeDefs = {};
        #types;
        constructor(registry, value, isContract) {
            super(registry, {
                types: 'Vec<PortableType>'
            }, value);
            const { lookups, names, params, types } = extractTypeInfo(this, this.types);
            this.#alias = extractAliases(params, isContract);
            this.#lookups = lookups;
            this.#names = names;
            this.#params = params;
            this.#types = types;
        }
        get names() {
            return Object.values(this.#names).sort();
        }
        get paramTypes() {
            return this.#params;
        }
        get types() {
            return this.getT('types');
        }
        register() {
            registerTypes(this, this.#lookups, this.#names, this.#params);
        }
        getName(lookupId) {
            return this.#names[this.#getLookupId(lookupId)];
        }
        getSiType(lookupId) {
            const found = (this.#types || this.types)[this.#getLookupId(lookupId)];
            if (!found) {
                throw new Error(`PortableRegistry: Unable to find type with lookupId ${lookupId.toString()}`);
            }
            return found.type;
        }
        getTypeDef(lookupId) {
            const lookupIndex = this.#getLookupId(lookupId);
            if (!this.#typeDefs[lookupIndex]) {
                const lookupName = this.#names[lookupIndex];
                const empty = {
                    info: exports.TypeDefInfo.DoNotConstruct,
                    lookupIndex,
                    lookupName,
                    type: this.registry.createLookupType(lookupIndex)
                };
                if (lookupName) {
                    this.#typeDefs[lookupIndex] = empty;
                }
                const extracted = this.#extract(this.getSiType(lookupId), lookupIndex);
                if (!lookupName) {
                    this.#typeDefs[lookupIndex] = empty;
                }
                Object.keys(extracted).forEach((k) => {
                    if (k !== 'lookupName' || extracted[k]) {
                        this.#typeDefs[lookupIndex][k] = extracted[k];
                    }
                });
                if (extracted.info === exports.TypeDefInfo.Plain) {
                    this.#typeDefs[lookupIndex].lookupNameRoot = this.#typeDefs[lookupIndex].lookupName;
                    delete this.#typeDefs[lookupIndex].lookupName;
                }
            }
            return this.#typeDefs[lookupIndex];
        }
        sanitizeField(name) {
            let nameField = null;
            let nameOrig = null;
            if (name.isSome) {
                nameField = util.stringCamelCase(name.unwrap());
                if (nameField.includes('#')) {
                    nameOrig = nameField;
                    nameField = nameOrig.replace(/#/g, '_');
                }
                else if (RESERVED.includes(nameField)) {
                    nameOrig = nameField;
                    nameField = `${nameField}_`;
                }
            }
            return [nameField, nameOrig];
        }
        #createSiDef(lookupId) {
            const typeDef = this.getTypeDef(lookupId);
            const lookupIndex = lookupId.toNumber();
            return [exports.TypeDefInfo.DoNotConstruct, exports.TypeDefInfo.Enum, exports.TypeDefInfo.Struct].includes(typeDef.info) && typeDef.lookupName
                ? {
                    docs: typeDef.docs,
                    info: exports.TypeDefInfo.Si,
                    lookupIndex,
                    lookupName: this.#names[lookupIndex],
                    type: this.registry.createLookupType(lookupId)
                }
                : typeDef;
        }
        #getLookupId(lookupId) {
            if (util.isString(lookupId)) {
                if (!this.registry.isLookupType(lookupId)) {
                    throw new Error(`PortableRegistry: Expected a lookup string type, found ${lookupId}`);
                }
                return parseInt(lookupId.replace('Lookup', ''), 10);
            }
            else if (util.isNumber(lookupId)) {
                return lookupId;
            }
            return lookupId.toNumber();
        }
        #extract(type, lookupIndex) {
            const namespace = type.path.join('::');
            let typeDef;
            const aliasType = this.#alias[lookupIndex] || getAliasPath(type);
            try {
                if (aliasType) {
                    typeDef = this.#extractAliasPath(lookupIndex, aliasType);
                }
                else {
                    switch (type.def.type) {
                        case 'Array':
                            typeDef = this.#extractArray(lookupIndex, type.def.asArray);
                            break;
                        case 'BitSequence':
                            typeDef = this.#extractBitSequence(lookupIndex, type.def.asBitSequence);
                            break;
                        case 'Compact':
                            typeDef = this.#extractCompact(lookupIndex, type.def.asCompact);
                            break;
                        case 'Composite':
                            typeDef = this.#extractComposite(lookupIndex, type, type.def.asComposite);
                            break;
                        case 'HistoricMetaCompat':
                            typeDef = this.#extractHistoric(lookupIndex, type.def.asHistoricMetaCompat);
                            break;
                        case 'Primitive':
                            typeDef = this.#extractPrimitive(lookupIndex, type);
                            break;
                        case 'Sequence':
                            typeDef = this.#extractSequence(lookupIndex, type.def.asSequence);
                            break;
                        case 'Tuple':
                            typeDef = this.#extractTuple(lookupIndex, type.def.asTuple);
                            break;
                        case 'Variant':
                            typeDef = this.#extractVariant(lookupIndex, type, type.def.asVariant);
                            break;
                        default: util.assertUnreachable(type.def.type);
                    }
                }
            }
            catch (error) {
                throw new Error(`PortableRegistry: ${lookupIndex}${namespace ? ` (${namespace})` : ''}: Error extracting ${util.stringify(type)}: ${error.message}`);
            }
            return util.objectSpread({
                docs: sanitizeDocs(type.docs),
                namespace
            }, typeDef);
        }
        #extractArray(_, { len, type }) {
            const length = len.toNumber();
            if (length > 2048) {
                throw new Error('Only support for [Type; <length>], where length <= 2048');
            }
            return withTypeString(this.registry, {
                info: exports.TypeDefInfo.VecFixed,
                length,
                sub: this.#createSiDef(type)
            });
        }
        #extractBitSequence(_, { bitOrderType, bitStoreType }) {
            const a = this.#createSiDef(bitOrderType);
            const b = this.#createSiDef(bitStoreType);
            const [bitOrder, bitStore] = BITVEC_NS.includes(a.namespace || '')
                ? [a, b]
                : [b, a];
            if (!bitOrder.namespace || !BITVEC_NS.includes(bitOrder.namespace)) {
                throw new Error(`Unexpected bitOrder found as ${bitOrder.namespace || '<unknown>'}`);
            }
            else if (bitStore.info !== exports.TypeDefInfo.Plain || bitStore.type !== 'u8') {
                throw new Error(`Only u8 bitStore is currently supported, found ${bitStore.type}`);
            }
            BITVEC_NS_LSB.includes(bitOrder.namespace);
            return {
                info: exports.TypeDefInfo.Plain,
                type: 'BitVec'
            };
        }
        #extractCompact(_, { type }) {
            return withTypeString(this.registry, {
                info: exports.TypeDefInfo.Compact,
                sub: this.#createSiDef(type)
            });
        }
        #extractComposite(lookupIndex, { params, path }, { fields }) {
            if (path.length) {
                const pathFirst = path[0].toString();
                const pathLast = path[path.length - 1].toString();
                if (path.length === 1 && pathFirst === 'BTreeMap') {
                    if (params.length !== 2) {
                        throw new Error(`BTreeMap requires 2 parameters, found ${params.length}`);
                    }
                    return withTypeString(this.registry, {
                        info: exports.TypeDefInfo.BTreeMap,
                        sub: params.map(({ type }) => this.#createSiDef(type.unwrap()))
                    });
                }
                else if (path.length === 1 && pathFirst === 'BTreeSet') {
                    if (params.length !== 1) {
                        throw new Error(`BTreeSet requires 1 parameter, found ${params.length}`);
                    }
                    return withTypeString(this.registry, {
                        info: exports.TypeDefInfo.BTreeSet,
                        sub: this.#createSiDef(params[0].type.unwrap())
                    });
                }
                else if (['Range', 'RangeInclusive'].includes(pathFirst)) {
                    if (params.length !== 1) {
                        throw new Error(`Range requires 1 parameter, found ${params.length}`);
                    }
                    return withTypeString(this.registry, {
                        info: pathFirst === 'Range'
                            ? exports.TypeDefInfo.Range
                            : exports.TypeDefInfo.RangeInclusive,
                        sub: this.#createSiDef(params[0].type.unwrap()),
                        type: pathFirst
                    });
                }
                else if (['WrapperKeepOpaque', 'WrapperOpaque'].includes(pathLast)) {
                    if (params.length !== 1) {
                        throw new Error(`WrapperOpaque requires 1 parameter, found ${params.length}`);
                    }
                    return withTypeString(this.registry, {
                        info: pathLast === 'WrapperKeepOpaque'
                            ? exports.TypeDefInfo.WrapperKeepOpaque
                            : exports.TypeDefInfo.WrapperOpaque,
                        sub: this.#createSiDef(params[0].type.unwrap()),
                        type: pathLast
                    });
                }
            }
            return PATHS_SET.some((p) => matchParts(p, path))
                ? this.#extractCompositeSet(lookupIndex, params, fields)
                : this.#extractFields(lookupIndex, fields);
        }
        #extractCompositeSet(_, params, fields) {
            if (params.length !== 1 || fields.length !== 1) {
                throw new Error('Set handling expects param/field as single entries');
            }
            return withTypeString(this.registry, {
                info: exports.TypeDefInfo.Set,
                length: this.registry.createTypeUnsafe(this.registry.createLookupType(fields[0].type), []).bitLength(),
                sub: this.getSiType(params[0].type.unwrap()).def.asVariant.variants.map(({ index, name }) => ({
                    index: index.toNumber(),
                    info: exports.TypeDefInfo.Plain,
                    name: name.toString(),
                    type: 'Null'
                }))
            });
        }
        #extractFields(lookupIndex, fields) {
            let isStruct = true;
            let isTuple = true;
            const count = fields.length;
            for (let f = 0; f < count; f++) {
                const { name } = fields[f];
                isStruct = isStruct && name.isSome;
                isTuple = isTuple && name.isNone;
            }
            if (!isTuple && !isStruct) {
                throw new Error('Invalid fields type detected, expected either Tuple (all unnamed) or Struct (all named)');
            }
            if (count === 0) {
                return {
                    info: exports.TypeDefInfo.Null,
                    type: 'Null'
                };
            }
            else if (isTuple && count === 1) {
                const typeDef = this.#createSiDef(fields[0].type);
                return util.objectSpread({}, typeDef, lookupIndex === -1
                    ? null
                    : {
                        lookupIndex,
                        lookupName: this.#names[lookupIndex],
                        lookupNameRoot: typeDef.lookupName
                    }, fields[0].typeName.isSome
                    ? { typeName: sanitize(fields[0].typeName.unwrap()) }
                    : null);
            }
            const [sub, alias] = this.#extractFieldsAlias(fields);
            return withTypeString(this.registry, util.objectSpread({
                info: isTuple
                    ? exports.TypeDefInfo.Tuple
                    : exports.TypeDefInfo.Struct,
                sub
            }, alias.size
                ? { alias }
                : null, lookupIndex === -1
                ? null
                : {
                    lookupIndex,
                    lookupName: this.#names[lookupIndex]
                }));
        }
        #extractFieldsAlias(fields) {
            const alias = new Map();
            const count = fields.length;
            const sub = new Array(count);
            for (let i = 0; i < count; i++) {
                const { docs, name, type, typeName } = fields[i];
                const typeDef = this.#createSiDef(type);
                if (name.isNone) {
                    sub[i] = typeDef;
                }
                else {
                    const [nameField, nameOrig] = this.sanitizeField(name);
                    if (nameField && nameOrig) {
                        alias.set(nameField, nameOrig);
                    }
                    sub[i] = util.objectSpread({
                        docs: sanitizeDocs(docs),
                        name: nameField
                    }, typeDef, typeName.isSome
                        ? { typeName: sanitize(typeName.unwrap()) }
                        : null);
                }
            }
            return [sub, alias];
        }
        #extractHistoric(_, type) {
            return util.objectSpread({
                displayName: type.toString(),
                isFromSi: true
            }, getTypeDef(type));
        }
        #extractPrimitive(_, type) {
            const typeStr = type.def.asPrimitive.type.toString();
            return {
                info: exports.TypeDefInfo.Plain,
                type: PRIMITIVE_ALIAS[typeStr] || typeStr.toLowerCase()
            };
        }
        #extractAliasPath(_, type) {
            return {
                info: exports.TypeDefInfo.Plain,
                type
            };
        }
        #extractSequence(lookupIndex, { type }) {
            const sub = this.#createSiDef(type);
            if (sub.type === 'u8') {
                return {
                    info: exports.TypeDefInfo.Plain,
                    type: 'Bytes'
                };
            }
            return withTypeString(this.registry, {
                info: exports.TypeDefInfo.Vec,
                lookupIndex,
                lookupName: this.#names[lookupIndex],
                sub
            });
        }
        #extractTuple(lookupIndex, ids) {
            if (ids.length === 0) {
                return {
                    info: exports.TypeDefInfo.Null,
                    type: 'Null'
                };
            }
            else if (ids.length === 1) {
                return this.getTypeDef(ids[0]);
            }
            const sub = ids.map((t) => this.#createSiDef(t));
            return withTypeString(this.registry, {
                info: exports.TypeDefInfo.Tuple,
                lookupIndex,
                lookupName: this.#names[lookupIndex],
                sub
            });
        }
        #extractVariant(lookupIndex, { params, path }, { variants }) {
            if (path.length) {
                const specialVariant = path[0].toString();
                if (specialVariant === 'Option') {
                    if (params.length !== 1) {
                        throw new Error(`Option requires 1 parameter, found ${params.length}`);
                    }
                    return withTypeString(this.registry, {
                        info: exports.TypeDefInfo.Option,
                        sub: this.#createSiDef(params[0].type.unwrap())
                    });
                }
                else if (specialVariant === 'Result') {
                    if (params.length !== 2) {
                        throw new Error(`Result requires 2 parameters, found ${params.length}`);
                    }
                    return withTypeString(this.registry, {
                        info: exports.TypeDefInfo.Result,
                        sub: params.map(({ type }, index) => util.objectSpread({
                            name: ['Ok', 'Error'][index]
                        }, this.#createSiDef(type.unwrap())))
                    });
                }
            }
            if (variants.length === 0) {
                return {
                    info: exports.TypeDefInfo.Null,
                    type: 'Null'
                };
            }
            return this.#extractVariantEnum(lookupIndex, variants);
        }
        #extractVariantEnum(lookupIndex, variants) {
            const sub = [];
            variants
                .slice()
                .sort((a, b) => a.index.cmp(b.index))
                .forEach(({ fields, index: bnIndex, name }) => {
                const index = bnIndex.toNumber();
                while (sub.length !== index) {
                    sub.push({
                        index: sub.length,
                        info: exports.TypeDefInfo.Null,
                        name: `__Unused${sub.length}`,
                        type: 'Null'
                    });
                }
                sub.push(util.objectSpread(this.#extractFields(-1, fields), {
                    index,
                    name: name.toString()
                }));
            });
            return withTypeString(this.registry, {
                info: exports.TypeDefInfo.Enum,
                lookupIndex,
                lookupName: this.#names[lookupIndex],
                sub
            });
        }
    }

    function convertType(key) {
        return (registry, { type }) => registry.createType('Si1TypeDef', {
            [key]: {
                type: type.toNumber()
            }
        });
    }
    function convertArray(registry, { len, type }) {
        return registry.createType('Si1TypeDef', {
            Array: {
                len,
                type: type.toNumber()
            }
        });
    }
    function convertBitSequence(registry, { bitOrderType, bitStoreType }) {
        return registry.createType('Si1TypeDef', {
            BitSequence: {
                bitOrderType: bitOrderType.toNumber(),
                bitStoreType: bitStoreType.toNumber()
            }
        });
    }
    const convertCompact = convertType('Compact');
    function convertComposite(registry, { fields }) {
        return registry.createType('Si1TypeDef', {
            Composite: {
                fields: convertFields(registry, fields)
            }
        });
    }
    function convertFields(registry, fields) {
        return fields.map(({ docs, name, type, typeName }) => registry.createType('Si1Field', {
            docs,
            name,
            type: type.toNumber(),
            typeName
        }));
    }
    function convertPhantom(registry, path) {
        console.warn(`Converting phantom type ${path.map((p) => p.toString()).join('::')} to empty tuple`);
        return registry.createType('Si1TypeDef', {
            Tuple: []
        });
    }
    function convertPrimitive(registry, prim) {
        return registry.createType('Si1TypeDef', {
            Primitive: prim.toString()
        });
    }
    const convertSequence = convertType('Sequence');
    function convertTuple(registry, types) {
        return registry.createType('Si1TypeDef', {
            Tuple: types.map((t) => t.toNumber())
        });
    }
    function convertVariant(registry, { variants }) {
        return registry.createType('Si1TypeDef', {
            Variant: {
                variants: variants.map(({ discriminant, docs, fields, name }, index) => registry.createType('Si1Variant', {
                    docs,
                    fields: convertFields(registry, fields),
                    index: discriminant.isSome
                        ? discriminant.unwrap().toNumber()
                        : index,
                    name
                }))
            }
        });
    }
    function convertDef(registry, { def, path }) {
        let result;
        switch (def.type) {
            case 'Array':
                result = convertArray(registry, def.asArray);
                break;
            case 'BitSequence':
                result = convertBitSequence(registry, def.asBitSequence);
                break;
            case 'Compact':
                result = convertCompact(registry, def.asCompact);
                break;
            case 'Composite':
                result = convertComposite(registry, def.asComposite);
                break;
            case 'Phantom':
                result = convertPhantom(registry, path);
                break;
            case 'Primitive':
                result = convertPrimitive(registry, def.asPrimitive);
                break;
            case 'Sequence':
                result = convertSequence(registry, def.asSequence);
                break;
            case 'Tuple':
                result = convertTuple(registry, def.asTuple);
                break;
            case 'Variant':
                result = convertVariant(registry, def.asVariant);
                break;
            default: util.assertUnreachable(def.type);
        }
        return result;
    }
    function toV1(registry, types) {
        return types.map((t, index) => registry.createType('PortableType', {
            id: index + 1,
            type: {
                def: convertDef(registry, t),
                docs: [],
                params: t.params.map((p) => registry.createType('Si1TypeParameter', {
                    type: p.toNumber()
                })),
                path: t.path.map((p) => p.toString())
            }
        }));
    }

    const packageInfo = { name: '@polkadot/types', path: (({ url: (typeof document === 'undefined' && typeof location === 'undefined' ? require('u' + 'rl').pathToFileURL(__filename).href : typeof document === 'undefined' ? location.href : (_documentCurrentScript && _documentCurrentScript.src || new URL('bundle-polkadot-types.js', document.baseURI).href)) }) && (typeof document === 'undefined' && typeof location === 'undefined' ? require('u' + 'rl').pathToFileURL(__filename).href : typeof document === 'undefined' ? location.href : (_documentCurrentScript && _documentCurrentScript.src || new URL('bundle-polkadot-types.js', document.baseURI).href))) ? new URL((typeof document === 'undefined' && typeof location === 'undefined' ? require('u' + 'rl').pathToFileURL(__filename).href : typeof document === 'undefined' ? location.href : (_documentCurrentScript && _documentCurrentScript.src || new URL('bundle-polkadot-types.js', document.baseURI).href))).pathname.substring(0, new URL((typeof document === 'undefined' && typeof location === 'undefined' ? require('u' + 'rl').pathToFileURL(__filename).href : typeof document === 'undefined' ? location.href : (_documentCurrentScript && _documentCurrentScript.src || new URL('bundle-polkadot-types.js', document.baseURI).href))).pathname.lastIndexOf('/') + 1) : 'auto', type: 'esm', version: '16.4.9' };

    function flattenUniq(list, result = []) {
        for (let i = 0, count = list.length; i < count; i++) {
            const entry = list[i];
            if (Array.isArray(entry)) {
                flattenUniq(entry, result);
            }
            else {
                result.push(entry);
            }
        }
        return [...new Set(result)];
    }

    function getSiName(lookup, type) {
        const typeDef = lookup.getTypeDef(type);
        return typeDef.lookupName || typeDef.type;
    }

    function extractSubSingle(_, { sub }) {
        const { lookupName, type } = sub;
        return extractTypes$1([lookupName || type]);
    }
    function extractSubArray(_, { sub }) {
        return extractTypes$1(sub.map(({ lookupName, type }) => lookupName || type));
    }
    function unhandled(type, { info }) {
        throw new Error(`Unhandled: Unable to create and validate type from ${type} (info=${exports.TypeDefInfo[info]})`);
    }
    const mapping = {
        [exports.TypeDefInfo.BTreeMap]: extractSubArray,
        [exports.TypeDefInfo.BTreeSet]: extractSubSingle,
        [exports.TypeDefInfo.Compact]: extractSubSingle,
        [exports.TypeDefInfo.DoNotConstruct]: unhandled,
        [exports.TypeDefInfo.Enum]: extractSubArray,
        [exports.TypeDefInfo.HashMap]: extractSubArray,
        [exports.TypeDefInfo.Int]: unhandled,
        [exports.TypeDefInfo.Linkage]: extractSubSingle,
        [exports.TypeDefInfo.Null]: unhandled,
        [exports.TypeDefInfo.Option]: extractSubSingle,
        [exports.TypeDefInfo.Plain]: (_, typeDef) => typeDef.lookupName || typeDef.type,
        [exports.TypeDefInfo.Range]: extractSubSingle,
        [exports.TypeDefInfo.RangeInclusive]: extractSubSingle,
        [exports.TypeDefInfo.Result]: extractSubArray,
        [exports.TypeDefInfo.Set]: extractSubArray,
        [exports.TypeDefInfo.Si]: unhandled,
        [exports.TypeDefInfo.Struct]: extractSubArray,
        [exports.TypeDefInfo.Tuple]: extractSubArray,
        [exports.TypeDefInfo.UInt]: unhandled,
        [exports.TypeDefInfo.Vec]: extractSubSingle,
        [exports.TypeDefInfo.VecFixed]: extractSubSingle,
        [exports.TypeDefInfo.WrapperKeepOpaque]: extractSubSingle,
        [exports.TypeDefInfo.WrapperOpaque]: extractSubSingle
    };
    function extractTypes$1(types) {
        const count = types.length;
        const result = new Array(count);
        for (let i = 0; i < count; i++) {
            const type = types[i];
            const typeDef = getTypeDef(type);
            result[i] = mapping[typeDef.info](type, typeDef);
        }
        return result;
    }

    const l$1 = util.logger('metadata');
    function validateTypes(registry, throwError, types) {
        const missing = flattenUniq(extractTypes$1(types))
            .filter((type) => !registry.hasType(type) &&
            !registry.isLookupType(type))
            .sort();
        if (missing.length !== 0) {
            const message = `Unknown types found, no types for ${missing.join(', ')}`;
            if (throwError) {
                throw new Error(message);
            }
            else {
                l$1.warn(message);
            }
        }
        return types;
    }

    function extractTypes(lookup, types) {
        return types.map(({ type }) => lookup.getTypeDef(type).type);
    }
    function extractFieldTypes(lookup, type) {
        return lookup.getSiType(type).def.asVariant.variants.map(({ fields }) => extractTypes(lookup, fields));
    }
    function getPalletNames({ lookup, pallets }) {
        return pallets.reduce((all, { calls, constants, events, storage }) => {
            all.push([extractTypes(lookup, constants)]);
            if (calls.isSome) {
                all.push(extractFieldTypes(lookup, calls.unwrap().type));
            }
            if (events.isSome) {
                all.push(extractFieldTypes(lookup, events.unwrap().type));
            }
            if (storage.isSome) {
                all.push(storage.unwrap().items.map(({ type }) => {
                    if (type.isPlain) {
                        return [lookup.getTypeDef(type.asPlain).type];
                    }
                    const { hashers, key, value } = type.asMap;
                    return hashers.length === 1
                        ? [
                            lookup.getTypeDef(value).type,
                            lookup.getTypeDef(key).type
                        ]
                        : [
                            lookup.getTypeDef(value).type,
                            ...lookup.getSiType(key).def.asTuple.map((t) => lookup.getTypeDef(t).type)
                        ];
                }));
            }
            return all;
        }, []);
    }
    function getUniqTypes(registry, meta, throwError) {
        return validateTypes(registry, throwError, flattenUniq(getPalletNames(meta)));
    }

    function trimDocs(docs) {
        const strings = docs.map((d) => d.toString().trim());
        const firstEmpty = strings.findIndex((d) => !d.length);
        return firstEmpty === -1
            ? strings
            : strings.slice(0, firstEmpty);
    }
    function toCallsOnly(registry, { extrinsic, lookup, pallets }) {
        return registry.createTypeUnsafe('MetadataLatest', [{
                extrinsic,
                lookup: {
                    types: lookup.types.map(({ id, type }) => registry.createTypeUnsafe('PortableType', [{
                            id,
                            type: util.objectSpread({}, type, { docs: trimDocs(type.docs) })
                        }]))
                },
                pallets: pallets.map(({ calls, index, name }) => ({
                    calls: registry.createTypeUnsafe('Option<PalletCallMetadataLatest>', [calls.unwrapOr(null)]),
                    index,
                    name
                }))
            }]).toJSON();
    }

    function unwrapStorageSi(type) {
        return type.isPlain
            ? type.asPlain
            : type.asMap.value;
    }
    function unwrapStorageType(registry, type, isOptional) {
        const outputType = getSiName(registry.lookup, unwrapStorageSi(type));
        return isOptional
            ? `Option<${outputType}>`
            : outputType;
    }

    function createClass(registry, type) {
        return createClassUnsafe(registry, type);
    }

    function createType(registry, type, ...params) {
        return createTypeUnsafe(registry, type, params);
    }

    function lazyVariants(lookup, { type }, getName, creator) {
        const result = {};
        const variants = lookup.getSiType(type).def.asVariant.variants;
        for (let i = 0, count = variants.length; i < count; i++) {
            util.lazyMethod(result, variants[i], creator, getName, i);
        }
        return result;
    }

    const emptyCheck = {
        extrinsic: {},
        payload: {}
    };

    const polkadot = {
        LimitParathreadCommits: emptyCheck,
        OnlyStakingAndClaims: emptyCheck,
        PrevalidateAttests: emptyCheck,
        RestrictFunctionality: emptyCheck,
        TransactionCallFilter: emptyCheck,
        ValidateDoubleVoteReports: emptyCheck
    };

    const shell = {
        DisallowSigned: emptyCheck
    };

    const statemint = {
        ChargeAssetTxPayment: {
            extrinsic: {
                tip: 'Compact<Balance>',
                assetId: 'TAssetConversion'
            },
            payload: {}
        }
    };

    const CheckMetadataHash = {
        extrinsic: {
            mode: 'u8'
        },
        payload: {
            metadataHash: 'Option<[u8;32]>'
        }
    };
    const CheckMortality = {
        extrinsic: {
            era: 'ExtrinsicEra'
        },
        payload: {
            blockHash: 'Hash'
        }
    };
    const ChargeTransactionPayment = {
        extrinsic: {
            tip: 'Compact<Balance>'
        },
        payload: {}
    };
    const substrate$1 = {
        ChargeTransactionPayment,
        CheckBlockGasLimit: emptyCheck,
        CheckEra: CheckMortality,
        CheckGenesis: {
            extrinsic: {},
            payload: {
                genesisHash: 'Hash'
            }
        },
        CheckMetadataHash,
        CheckMortality,
        CheckNonZeroSender: emptyCheck,
        CheckNonce: {
            extrinsic: {
                nonce: 'Compact<Index>'
            },
            payload: {}
        },
        CheckSpecVersion: {
            extrinsic: {},
            payload: {
                specVersion: 'u32'
            }
        },
        CheckTxVersion: {
            extrinsic: {},
            payload: {
                transactionVersion: 'u32'
            }
        },
        CheckVersion: {
            extrinsic: {},
            payload: {
                specVersion: 'u32'
            }
        },
        CheckWeight: emptyCheck,
        LockStakingStatus: emptyCheck,
        SkipCheckIfFeeless: ChargeTransactionPayment,
        ValidateEquivocationReport: emptyCheck,
        WeightReclaim: emptyCheck
    };

    const allExtensions = util.objectSpread({}, substrate$1, polkadot, shell, statemint);
    const fallbackExtensions = [
        'CheckVersion',
        'CheckGenesis',
        'CheckEra',
        'CheckNonce',
        'CheckWeight',
        'ChargeTransactionPayment',
        'CheckBlockGasLimit'
    ];
    function findUnknownExtensions(extensions, userExtensions = {}) {
        const names = [...Object.keys(allExtensions), ...Object.keys(userExtensions)];
        return extensions.filter((k) => !names.includes(k));
    }
    function expandExtensionTypes(extensions, type, userExtensions = {}) {
        return extensions
            .map((k) => userExtensions[k] || allExtensions[k])
            .filter((info) => !!info)
            .reduce((result, info) => util.objectSpread(result, info[type]), {});
    }

    function decodeEvent(registry, value) {
        if (!value?.length) {
            return { DataType: Null };
        }
        const index = value.subarray(0, 2);
        return {
            DataType: registry.findMetaEvent(index),
            value: {
                data: value.subarray(2),
                index
            }
        };
    }
    class GenericEventData extends Tuple {
        #meta;
        #method;
        #names = null;
        #section;
        #typeDef;
        constructor(registry, value, meta, section = '<unknown>', method = '<unknown>') {
            const fields = meta?.fields || [];
            super(registry, fields.map(({ type }) => registry.createLookupType(type)), value);
            this.#meta = meta;
            this.#method = method;
            this.#section = section;
            this.#typeDef = fields.map(({ type }) => registry.lookup.getTypeDef(type));
            const names = fields
                .map(({ name }) => registry.lookup.sanitizeField(name)[0])
                .filter((n) => !!n);
            if (names.length === fields.length) {
                this.#names = names;
                util.objectProperties(this, names, (_, i) => this[i]);
            }
        }
        get meta() {
            return this.#meta;
        }
        get method() {
            return this.#method;
        }
        get names() {
            return this.#names;
        }
        get section() {
            return this.#section;
        }
        get typeDef() {
            return this.#typeDef;
        }
        toHuman(isExtended, disableAscii) {
            if (this.#names !== null) {
                const json = {};
                for (let i = 0, count = this.#names.length; i < count; i++) {
                    json[this.#names[i]] = this[i].toHuman(isExtended, disableAscii);
                }
                return json;
            }
            return super.toHuman(isExtended);
        }
    }
    class GenericEvent extends Struct {
        constructor(registry, _value) {
            const { DataType, value } = decodeEvent(registry, _value);
            super(registry, {
                index: 'EventId',
                data: DataType
            }, value);
        }
        get data() {
            return this.getT('data');
        }
        get index() {
            return this.getT('index');
        }
        get meta() {
            return this.data.meta;
        }
        get method() {
            return this.data.method;
        }
        get section() {
            return this.data.section;
        }
        get typeDef() {
            return this.data.typeDef;
        }
        toHuman(isExpanded, disableAscii) {
            return util.objectSpread({
                method: this.method,
                section: this.section
            }, isExpanded
                ? { docs: this.meta.docs.map((d) => d.toString()) }
                : null, super.toHuman(isExpanded, disableAscii));
        }
    }

    const BIT_SIGNED = 0b10000000;
    const BIT_UNSIGNED = 0;
    const EMPTY_U8A = new Uint8Array();
    const IMMORTAL_ERA = new Uint8Array([0]);
    const UNMASK_VERSION = 0b01111111;
    const DEFAULT_PREAMBLE = 'bare';
    const LATEST_EXTRINSIC_VERSION = 4;
    const VERSION_MASK = 0b00111111;
    const TYPE_MASK = 0b11000000;
    const BARE_EXTRINSIC = 0b00000000;
    const GENERAL_EXTRINSIC = 0b01000000;
    const LOWEST_SUPPORTED_EXTRINSIC_FORMAT_VERSION = 4;

    const VERSIONS$1 = [
        'ExtrinsicUnknown',
        'ExtrinsicUnknown',
        'ExtrinsicUnknown',
        'ExtrinsicUnknown',
        'ExtrinsicV4',
        'ExtrinsicV5'
    ];
    const PREAMBLE = {
        bare: 'ExtrinsicV5',
        general: 'GeneralExtrinsic'
    };
    const PreambleMask = {
        bare: BARE_EXTRINSIC,
        general: GENERAL_EXTRINSIC
    };
    const preambleUnMask = {
        0: 'bare',
        64: 'general'
    };
    function newFromValue(registry, value, version, preamble) {
        if (value instanceof GenericExtrinsic) {
            return value.unwrap();
        }
        const isSigned = (version & BIT_SIGNED) === BIT_SIGNED;
        const type = (version & VERSION_MASK) === 5 ? PREAMBLE[preamble] : VERSIONS$1[version & VERSION_MASK] || VERSIONS$1[0];
        return registry.createTypeUnsafe(type, [value, { isSigned, version }]);
    }
    function decodeExtrinsic(registry, value, version = LOWEST_SUPPORTED_EXTRINSIC_FORMAT_VERSION, preamble = DEFAULT_PREAMBLE) {
        if (util.isU8a(value) || Array.isArray(value) || util.isHex(value)) {
            return decodeU8a$5(registry, util.u8aToU8a(value), version, preamble);
        }
        else if (value instanceof registry.createClassUnsafe('Call')) {
            return newFromValue(registry, { method: value }, version, preamble);
        }
        return newFromValue(registry, value, version, preamble);
    }
    function decodeU8a$5(registry, value, version, preamble) {
        if (!value.length) {
            return newFromValue(registry, new Uint8Array(), version, preamble);
        }
        const [offset, length] = util.compactFromU8a(value);
        const total = offset + length.toNumber();
        if (total > value.length) {
            throw new Error(`Extrinsic: length less than remainder, expected at least ${total}, found ${value.length}`);
        }
        const data = value.subarray(offset, total);
        const unmaskedPreamble = data[0] & TYPE_MASK;
        if (preambleUnMask[`${unmaskedPreamble}`] === 'general') {
            return newFromValue(registry, value, data[0], preambleUnMask[`${unmaskedPreamble}`] || preamble);
        }
        else {
            return newFromValue(registry, data.subarray(1), data[0], preambleUnMask[`${unmaskedPreamble}`] || preamble);
        }
    }
    class ExtrinsicBase extends AbstractBase {
        #preamble;
        constructor(registry, value, initialU8aLength, preamble) {
            super(registry, value, initialU8aLength);
            const signKeys = Object.keys(registry.getSignedExtensionTypes());
            if (this.version === 5 && preamble !== 'general') {
                const getter = (key) => this.inner.signature[key];
                for (let i = 0, count = signKeys.length; i < count; i++) {
                    util.objectProperty(this, signKeys[i], getter);
                }
            }
            const unmaskedPreamble = this.type & TYPE_MASK;
            this.#preamble = preamble || preambleUnMask[`${unmaskedPreamble}`];
        }
        isGeneral() {
            return this.#preamble === 'general';
        }
        get args() {
            return this.method.args;
        }
        get argsDef() {
            return this.method.argsDef;
        }
        get callIndex() {
            return this.method.callIndex;
        }
        get data() {
            return this.method.data;
        }
        get era() {
            return this.isGeneral()
                ? this.inner.era
                : this.inner.signature.era;
        }
        get encodedLength() {
            return this.toU8a().length;
        }
        get isSigned() {
            return this.isGeneral()
                ? false
                : this.inner.signature.isSigned;
        }
        get length() {
            return this.toU8a(true).length;
        }
        get meta() {
            return this.method.meta;
        }
        get method() {
            return this.inner.method;
        }
        get nonce() {
            return this.isGeneral()
                ? this.inner.nonce
                : this.inner.signature.nonce;
        }
        get signature() {
            if (this.isGeneral()) {
                throw new Error('Extrinsic: GeneralExtrinsic does not have signature implemented');
            }
            return this.inner.signature.signature;
        }
        get signer() {
            if (this.isGeneral()) {
                throw new Error('Extrinsic: GeneralExtrinsic does not have signer implemented');
            }
            return this.inner.signature.signer;
        }
        get tip() {
            return this.isGeneral()
                ? this.inner.tip
                : this.inner.signature.tip;
        }
        get assetId() {
            return this.isGeneral()
                ? this.inner.assetId
                : this.inner.signature.assetId;
        }
        get metadataHash() {
            return this.isGeneral()
                ? this.inner.metadataHash
                : this.inner.signature.metadataHash;
        }
        get mode() {
            return this.isGeneral()
                ? this.inner.mode
                : this.inner.signature.mode;
        }
        get type() {
            return this.inner.version;
        }
        get inner() {
            return this.unwrap();
        }
        get version() {
            if (this.type <= LOWEST_SUPPORTED_EXTRINSIC_FORMAT_VERSION) {
                return this.type | (this.isSigned ? BIT_SIGNED : BIT_UNSIGNED);
            }
            else {
                if (this.isSigned) {
                    throw new Error('Signed Extrinsics are currently only available for ExtrinsicV4');
                }
                return this.type | (this.isGeneral() ? PreambleMask.general : PreambleMask.bare);
            }
        }
        is(other) {
            return this.method.is(other);
        }
        unwrap() {
            return super.unwrap();
        }
    }
    class GenericExtrinsic extends ExtrinsicBase {
        #hashCache;
        static LATEST_EXTRINSIC_VERSION = LATEST_EXTRINSIC_VERSION;
        constructor(registry, value, { preamble, version } = {}) {
            const versionsLength = registry.metadata.extrinsic.versions.length;
            const supportedVersion = versionsLength ? registry.metadata.extrinsic.versions[0] : undefined;
            super(registry, decodeExtrinsic(registry, value, version || supportedVersion, preamble), undefined, preamble);
        }
        get hash() {
            if (!this.#hashCache) {
                this.#hashCache = super.hash;
            }
            return this.#hashCache;
        }
        addSignature(signer, signature, payload) {
            this.inner.addSignature(signer, signature, payload);
            this.#hashCache = undefined;
            return this;
        }
        inspect() {
            const encoded = util.u8aConcat(...this.toU8aInner());
            return {
                inner: this.isSigned
                    ? this.inner.inspect().inner
                    : this.inner.method.inspect().inner,
                outer: [util.compactToU8a(encoded.length), new Uint8Array([this.version])]
            };
        }
        sign(account, options) {
            this.inner.sign(account, options);
            this.#hashCache = undefined;
            return this;
        }
        signFake(signer, options) {
            this.inner.signFake(signer, options);
            this.#hashCache = undefined;
            return this;
        }
        toHex(isBare) {
            return util.u8aToHex(this.toU8a(isBare));
        }
        toHuman(isExpanded, disableAscii) {
            return util.objectSpread({}, {
                isSigned: this.isSigned,
                method: this.method.toHuman(isExpanded, disableAscii)
            }, this.isSigned
                ? {
                    assetId: this.assetId ? this.assetId.toHuman(isExpanded, disableAscii) : null,
                    era: this.era.toHuman(isExpanded, disableAscii),
                    metadataHash: this.metadataHash ? this.metadataHash.toHex() : null,
                    mode: this.mode ? this.mode.toHuman() : null,
                    nonce: this.nonce.toHuman(isExpanded, disableAscii),
                    signature: this.signature.toHex(),
                    signer: this.signer.toHuman(isExpanded, disableAscii),
                    tip: this.tip.toHuman(isExpanded, disableAscii)
                }
                : null);
        }
        toJSON() {
            return this.toHex();
        }
        toRawType() {
            return 'Extrinsic';
        }
        toU8a(isBare) {
            const encoded = util.u8aConcat(...this.toU8aInner());
            return isBare
                ? encoded
                : util.compactAddLength(encoded);
        }
        toU8aInner() {
            return [
                new Uint8Array([this.version]),
                this.inner.toU8a()
            ];
        }
    }

    function getTrailingZeros(period) {
        const binary = period.toString(2);
        let index = 0;
        while (binary[binary.length - 1 - index] === '0') {
            index++;
        }
        return index;
    }
    function decodeMortalEra(registry, value) {
        if (util.isU8a(value) || util.isHex(value) || Array.isArray(value)) {
            return decodeMortalU8a(registry, util.u8aToU8a(value));
        }
        else if (!value) {
            return [new u64(registry), new u64(registry)];
        }
        else if (util.isObject(value)) {
            return decodeMortalObject(registry, value);
        }
        throw new Error('Invalid data passed to Mortal era');
    }
    function decodeMortalObject(registry, value) {
        const { current, period } = value;
        let calPeriod = Math.pow(2, Math.ceil(Math.log2(period)));
        calPeriod = Math.min(Math.max(calPeriod, 4), 1 << 16);
        const phase = current % calPeriod;
        const quantizeFactor = Math.max(calPeriod >> 12, 1);
        const quantizedPhase = phase / quantizeFactor * quantizeFactor;
        return [new u64(registry, calPeriod), new u64(registry, quantizedPhase)];
    }
    function decodeMortalU8a(registry, value) {
        if (value.length === 0) {
            return [new u64(registry), new u64(registry)];
        }
        const first = util.u8aToBn(value.subarray(0, 1)).toNumber();
        const second = util.u8aToBn(value.subarray(1, 2)).toNumber();
        const encoded = first + (second << 8);
        const period = 2 << (encoded % (1 << 4));
        const quantizeFactor = Math.max(period >> 12, 1);
        const phase = (encoded >> 4) * quantizeFactor;
        if (period < 4 || phase >= period) {
            throw new Error('Invalid data passed to Mortal era');
        }
        return [new u64(registry, period), new u64(registry, phase)];
    }
    function decodeExtrinsicEra(value = new Uint8Array()) {
        if (util.isU8a(value)) {
            return (!value.length || value[0] === 0)
                ? new Uint8Array([0])
                : new Uint8Array([1, value[0], value[1]]);
        }
        else if (!value) {
            return new Uint8Array([0]);
        }
        else if (value instanceof GenericExtrinsicEra) {
            return decodeExtrinsicEra(value.toU8a());
        }
        else if (util.isHex(value)) {
            return decodeExtrinsicEra(util.hexToU8a(value));
        }
        else if (util.isObject(value)) {
            const entries = Object.entries(value).map(([k, v]) => [k.toLowerCase(), v]);
            const mortal = entries.find(([k]) => k.toLowerCase() === 'mortalera');
            const immortal = entries.find(([k]) => k.toLowerCase() === 'immortalera');
            return mortal
                ? { MortalEra: mortal[1] }
                : immortal
                    ? { ImmortalEra: immortal[1] }
                    : { MortalEra: value };
        }
        throw new Error('Invalid data passed to Era');
    }
    class ImmortalEra extends Raw {
        constructor(registry, _value) {
            super(registry, IMMORTAL_ERA);
        }
    }
    class MortalEra extends Tuple {
        constructor(registry, value) {
            super(registry, {
                period: u64,
                phase: u64
            }, decodeMortalEra(registry, value));
        }
        get encodedLength() {
            return 2 | 0;
        }
        get period() {
            return this[0];
        }
        get phase() {
            return this[1];
        }
        toHuman() {
            return {
                period: util.formatNumber(this.period),
                phase: util.formatNumber(this.phase)
            };
        }
        toJSON() {
            return this.toHex();
        }
        toU8a(_isBare) {
            const period = this.period.toNumber();
            const encoded = Math.min(15, Math.max(1, getTrailingZeros(period) - 1)) + ((this.phase.toNumber() / Math.max(period >> 12, 1)) << 4);
            return new Uint8Array([
                encoded & 0xff,
                encoded >> 8
            ]);
        }
        birth(current) {
            const phase = this.phase.toNumber();
            const period = this.period.toNumber();
            return (~~((Math.max(util.bnToBn(current).toNumber(), phase) - phase) / period) * period) + phase;
        }
        death(current) {
            return this.birth(current) + this.period.toNumber();
        }
    }
    class GenericExtrinsicEra extends Enum {
        constructor(registry, value) {
            super(registry, {
                ImmortalEra,
                MortalEra
            }, decodeExtrinsicEra(value));
        }
        get encodedLength() {
            return this.isImmortalEra
                ? this.asImmortalEra.encodedLength
                : this.asMortalEra.encodedLength;
        }
        get asImmortalEra() {
            if (!this.isImmortalEra) {
                throw new Error(`Cannot convert '${this.type}' via asImmortalEra`);
            }
            return this.inner;
        }
        get asMortalEra() {
            if (!this.isMortalEra) {
                throw new Error(`Cannot convert '${this.type}' via asMortalEra`);
            }
            return this.inner;
        }
        get isImmortalEra() {
            return this.index === 0;
        }
        get isMortalEra() {
            return this.index > 0;
        }
        toU8a(isBare) {
            return this.isMortalEra
                ? this.asMortalEra.toU8a(isBare)
                : this.asImmortalEra.toU8a(isBare);
        }
    }

    const VERSIONS = [
        'ExtrinsicPayloadUnknown',
        'ExtrinsicPayloadUnknown',
        'ExtrinsicPayloadUnknown',
        'ExtrinsicPayloadUnknown',
        'ExtrinsicPayloadV4',
        'ExtrinsicPayloadV5'
    ];
    const PREAMBLES = {
        bare: 'ExtrinsicPayloadV5',
        general: 'ExtrinsicPayloadV5'
    };
    function decodeAssetId(registry, payload) {
        const maybeAsset = payload?.assetId;
        if (maybeAsset && util.isHex(maybeAsset)) {
            const assetId = registry.createType('TAssetConversion', util.hexToU8a(maybeAsset));
            if (maybeAsset === '0x00' || maybeAsset === '0x01' + assetId.toHex().slice(2)) {
                return {
                    ...payload,
                    assetId: assetId.toJSON()
                };
            }
        }
        return payload;
    }
    function decodeExtrinsicPayload(registry, value, version = LATEST_EXTRINSIC_VERSION, preamble = DEFAULT_PREAMBLE) {
        if (value instanceof GenericExtrinsicPayload) {
            return value.unwrap();
        }
        const extVersion = version === 5 ? PREAMBLES[preamble] : VERSIONS[version] || VERSIONS[0];
        const payload = decodeAssetId(registry, value);
        return registry.createTypeUnsafe(extVersion, [payload, { version }]);
    }
    class GenericExtrinsicPayload extends AbstractBase {
        constructor(registry, value, { preamble, version } = {}) {
            super(registry, decodeExtrinsicPayload(registry, value, version, preamble));
        }
        get blockHash() {
            return this.inner.blockHash;
        }
        get era() {
            return this.inner.era;
        }
        get genesisHash() {
            return this.inner.genesisHash || this.registry.createTypeUnsafe('Hash', []);
        }
        get method() {
            return this.inner.method;
        }
        get nonce() {
            return this.inner.nonce;
        }
        get specVersion() {
            return this.inner.specVersion || this.registry.createTypeUnsafe('u32', []);
        }
        get tip() {
            return this.inner.tip || this.registry.createTypeUnsafe('Compact<Balance>', []);
        }
        get transactionVersion() {
            return this.inner.transactionVersion || this.registry.createTypeUnsafe('u32', []);
        }
        get assetId() {
            return this.inner.assetId;
        }
        get metadataHash() {
            return this.inner.metadataHash;
        }
        eq(other) {
            return this.inner.eq(other);
        }
        sign(signerPair) {
            const signature = this.inner.sign(signerPair);
            return {
                signature: util.u8aToHex(signature)
            };
        }
        toHuman(isExtended, disableAscii) {
            return this.inner.toHuman(isExtended, disableAscii);
        }
        toJSON() {
            return this.toHex();
        }
        toRawType() {
            return 'ExtrinsicPayload';
        }
        toString() {
            return this.toHex();
        }
        toU8a(isBare) {
            return super.toU8a(isBare ? { method: true } : false);
        }
    }

    class GenericExtrinsicPayloadUnknown extends Struct {
        constructor(registry, _value, { version = 0 } = {}) {
            super(registry, {});
            throw new Error(`Unsupported extrinsic payload version ${version}`);
        }
    }

    class GenericExtrinsicUnknown extends Struct {
        constructor(registry, _value, { isSigned = false, version = 0 } = {}) {
            super(registry, {});
            throw new Error(`Unsupported ${isSigned ? '' : 'un'}signed extrinsic version ${version & UNMASK_VERSION}`);
        }
    }

    const knownTypes = {
        address: 'Address',
        assetId: 'Option<TAssetConversion>',
        blockHash: 'Hash',
        blockNumber: 'BlockNumber',
        era: 'ExtrinsicEra',
        genesisHash: 'Hash',
        metadataHash: 'Option<[u8;32]>',
        method: 'Call',
        mode: 'u8',
        nonce: 'Compact<Index>',
        runtimeVersion: 'RuntimeVersion',
        signedExtensions: 'Vec<Text>',
        tip: 'Compact<Balance>',
        version: 'u8'
    };
    class GenericSignerPayload extends Struct {
        #extraTypes;
        constructor(registry, value) {
            const extensionTypes = util.objectSpread({}, registry.getSignedExtensionTypes(), registry.getSignedExtensionExtra());
            super(registry, util.objectSpread({}, extensionTypes, knownTypes, { withSignedTransaction: 'bool' }), value);
            this.#extraTypes = {};
            const getter = (key) => this.get(key);
            for (const [key, type] of Object.entries(extensionTypes)) {
                if (!knownTypes[key]) {
                    this.#extraTypes[key] = type;
                }
                util.objectProperty(this, key, getter);
            }
        }
        get address() {
            return this.getT('address');
        }
        get blockHash() {
            return this.getT('blockHash');
        }
        get blockNumber() {
            return this.getT('blockNumber');
        }
        get era() {
            return this.getT('era');
        }
        get genesisHash() {
            return this.getT('genesisHash');
        }
        get method() {
            return this.getT('method');
        }
        get nonce() {
            return this.getT('nonce');
        }
        get runtimeVersion() {
            return this.getT('runtimeVersion');
        }
        get signedExtensions() {
            return this.getT('signedExtensions');
        }
        get tip() {
            return this.getT('tip');
        }
        get assetId() {
            return this.getT('assetId');
        }
        get version() {
            return this.getT('version');
        }
        get mode() {
            return this.getT('mode');
        }
        get metadataHash() {
            return this.getT('metadataHash');
        }
        get withSignedTransaction() {
            const val = this.getT('withSignedTransaction');
            return val.isTrue;
        }
        toPayload() {
            const result = {};
            const keys = Object.keys(this.#extraTypes);
            for (let i = 0, count = keys.length; i < count; i++) {
                const key = keys[i];
                const value = this.getT(key);
                if (!(value instanceof Option) || value.isSome) {
                    result[key] = value.toJSON();
                }
            }
            return util.objectSpread(result, {
                address: this.address.toString(),
                assetId: this.assetId && this.assetId.isSome ? this.assetId.toHex() : null,
                blockHash: this.blockHash.toHex(),
                blockNumber: this.blockNumber.toHex(),
                era: this.era.toHex(),
                genesisHash: this.genesisHash.toHex(),
                metadataHash: this.metadataHash.isSome ? this.metadataHash.toHex() : null,
                method: this.method.toHex(),
                mode: this.mode.toNumber(),
                nonce: this.nonce.toHex(),
                signedExtensions: this.signedExtensions.map((e) => e.toString()),
                specVersion: this.runtimeVersion.specVersion.toHex(),
                tip: this.tip.toHex(),
                transactionVersion: this.runtimeVersion.transactionVersion.toHex(),
                version: this.version.toNumber(),
                withSignedTransaction: this.withSignedTransaction
            });
        }
        toRaw() {
            const payload = this.toPayload();
            const data = util.u8aToHex(this.registry
                .createTypeUnsafe('ExtrinsicPayload', [payload, { version: payload.version }])
                .toU8a({ method: true }));
            return {
                address: payload.address,
                data,
                type: 'payload'
            };
        }
    }

    const EXTRINSIC_VERSION$1 = 4;
    class GenericExtrinsicV4 extends Struct {
        constructor(registry, value, { isSigned } = {}) {
            super(registry, {
                signature: 'ExtrinsicSignatureV4',
                method: 'Call'
            }, GenericExtrinsicV4.decodeExtrinsic(registry, value, isSigned));
        }
        static decodeExtrinsic(registry, value, isSigned = false) {
            if (value instanceof GenericExtrinsicV4) {
                return value;
            }
            else if (value instanceof registry.createClassUnsafe('Call')) {
                return { method: value };
            }
            else if (util.isU8a(value)) {
                const signature = registry.createTypeUnsafe('ExtrinsicSignatureV4', [value, { isSigned }]);
                const method = registry.createTypeUnsafe('Call', [value.subarray(signature.encodedLength)]);
                return {
                    method,
                    signature
                };
            }
            return value || {};
        }
        get encodedLength() {
            return this.toU8a().length;
        }
        get method() {
            return this.getT('method');
        }
        get signature() {
            return this.getT('signature');
        }
        get version() {
            return EXTRINSIC_VERSION$1;
        }
        addSignature(signer, signature, payload) {
            this.signature.addSignature(signer, signature, payload);
            return this;
        }
        sign(account, options) {
            this.signature.sign(this.method, account, options);
            return this;
        }
        signFake(signer, options) {
            this.signature.signFake(this.method, signer, options);
            return this;
        }
    }

    function sign(_registry, signerPair, u8a, options) {
        const encoded = u8a.length > 256
            ? utilCrypto.blake2AsU8a(u8a)
            : u8a;
        return signerPair.sign(encoded, options);
    }

    class GenericExtrinsicPayloadV4 extends Struct {
        #signOptions;
        constructor(registry, value) {
            super(registry, util.objectSpread({ method: 'Bytes' }, registry.getSignedExtensionTypes(), registry.getSignedExtensionExtra()), decodeAssetId(registry, value));
            this.#signOptions = {
                withType: registry.createTypeUnsafe('ExtrinsicSignature', []) instanceof Enum
            };
        }
        inspect() {
            return super.inspect({ method: true });
        }
        get blockHash() {
            return this.getT('blockHash');
        }
        get era() {
            return this.getT('era');
        }
        get genesisHash() {
            return this.getT('genesisHash');
        }
        get method() {
            return this.getT('method');
        }
        get nonce() {
            return this.getT('nonce');
        }
        get specVersion() {
            return this.getT('specVersion');
        }
        get tip() {
            return this.getT('tip');
        }
        get transactionVersion() {
            return this.getT('transactionVersion');
        }
        get assetId() {
            return this.getT('assetId');
        }
        get metadataHash() {
            return this.getT('metadataHash');
        }
        sign(signerPair) {
            return sign(this.registry, signerPair, this.toU8a({ method: true }), this.#signOptions);
        }
    }

    const FAKE_SIGNATURE = new Uint8Array(256).fill(1);
    function toAddress(registry, address) {
        return registry.createTypeUnsafe('Address', [util.isU8a(address) ? util.u8aToHex(address) : address]);
    }
    class GenericExtrinsicSignatureV4 extends Struct {
        #signKeys;
        constructor(registry, value, { isSigned } = {}) {
            const signTypes = registry.getSignedExtensionTypes();
            super(registry, util.objectSpread(
            { signer: 'Address', signature: 'ExtrinsicSignature' }, signTypes), GenericExtrinsicSignatureV4.decodeExtrinsicSignature(value, isSigned));
            this.#signKeys = Object.keys(signTypes);
            util.objectProperties(this, this.#signKeys, (k) => this.get(k));
        }
        static decodeExtrinsicSignature(value, isSigned = false) {
            if (!value) {
                return EMPTY_U8A;
            }
            else if (value instanceof GenericExtrinsicSignatureV4) {
                return value;
            }
            return isSigned
                ? value
                : EMPTY_U8A;
        }
        get encodedLength() {
            return this.isSigned
                ? super.encodedLength
                : 0;
        }
        get isSigned() {
            return !this.signature.isEmpty;
        }
        get era() {
            return this.getT('era');
        }
        get nonce() {
            return this.getT('nonce');
        }
        get signature() {
            return (this.multiSignature.value || this.multiSignature);
        }
        get multiSignature() {
            return this.getT('signature');
        }
        get signer() {
            return this.getT('signer');
        }
        get tip() {
            return this.getT('tip');
        }
        get assetId() {
            return this.getT('assetId');
        }
        get mode() {
            return this.getT('mode');
        }
        get metadataHash() {
            return this.getT('metadataHash');
        }
        _injectSignature(signer, signature, payload) {
            for (let i = 0, count = this.#signKeys.length; i < count; i++) {
                const k = this.#signKeys[i];
                const v = payload.get(k);
                if (!util.isUndefined(v)) {
                    this.set(k, v);
                }
            }
            this.set('signer', signer);
            this.set('signature', signature);
            return this;
        }
        addSignature(signer, signature, payload) {
            return this._injectSignature(toAddress(this.registry, signer), this.registry.createTypeUnsafe('ExtrinsicSignature', [signature]), new GenericExtrinsicPayloadV4(this.registry, payload));
        }
        createPayload(method, options) {
            const { era, runtimeVersion: { specVersion, transactionVersion } } = options;
            return new GenericExtrinsicPayloadV4(this.registry, util.objectSpread({}, options, {
                era: era || IMMORTAL_ERA,
                method: method.toHex(),
                specVersion,
                transactionVersion
            }));
        }
        sign(method, account, options) {
            if (!account?.addressRaw) {
                throw new Error(`Expected a valid keypair for signing, found ${util.stringify(account)}`);
            }
            const payload = this.createPayload(method, options);
            return this._injectSignature(toAddress(this.registry, account.addressRaw), this.registry.createTypeUnsafe('ExtrinsicSignature', [payload.sign(account)]), payload);
        }
        signFake(method, address, options) {
            if (!address) {
                throw new Error(`Expected a valid address for signing, found ${util.stringify(address)}`);
            }
            const payload = this.createPayload(method, options);
            return this._injectSignature(toAddress(this.registry, address), this.registry.createTypeUnsafe('ExtrinsicSignature', [FAKE_SIGNATURE]), payload);
        }
        toU8a(isBare) {
            return this.isSigned
                ? super.toU8a(isBare)
                : EMPTY_U8A;
        }
    }

    const EXTRINSIC_VERSION = 5;
    class GenericExtrinsicV5 extends Struct {
        constructor(registry, value, { isSigned } = {}) {
            super(registry, {
                signature: 'ExtrinsicSignatureV5',
                method: 'Call'
            }, GenericExtrinsicV5.decodeExtrinsic(registry, value, isSigned));
        }
        static decodeExtrinsic(registry, value, isSigned = false) {
            if (value instanceof GenericExtrinsicV5) {
                return value;
            }
            else if (value instanceof registry.createClassUnsafe('Call')) {
                return { method: value };
            }
            else if (util.isU8a(value)) {
                const signature = registry.createTypeUnsafe('ExtrinsicSignatureV5', [value, { isSigned }]);
                const method = registry.createTypeUnsafe('Call', [value.subarray(signature.encodedLength)]);
                return {
                    method,
                    signature
                };
            }
            return value || {};
        }
        get encodedLength() {
            return this.toU8a().length;
        }
        get method() {
            return this.getT('method');
        }
        get signature() {
            return this.getT('signature');
        }
        get version() {
            return EXTRINSIC_VERSION;
        }
        get preamble() {
            return this.getT('preamble');
        }
        addSignature(_signer, _signature, _payload) {
            throw new Error('Extrinsic: ExtrinsicV5 does not include signing support');
        }
        sign(_account, _options) {
            throw new Error('Extrinsic: ExtrinsicV5 does not include signing support');
        }
        signFake(_signer, _options) {
            throw new Error('Extrinsic: ExtrinsicV5 does not include signing support');
        }
    }

    class GenericExtrinsicPayloadV5 extends Struct {
        constructor(registry, value) {
            super(registry, util.objectSpread({ method: 'Bytes' }, registry.getSignedExtensionTypes(), registry.getSignedExtensionExtra()), value);
        }
        inspect() {
            return super.inspect({ method: true });
        }
        get blockHash() {
            return this.getT('blockHash');
        }
        get era() {
            return this.getT('era');
        }
        get genesisHash() {
            return this.getT('genesisHash');
        }
        get method() {
            return this.getT('method');
        }
        get nonce() {
            return this.getT('nonce');
        }
        get specVersion() {
            return this.getT('specVersion');
        }
        get tip() {
            return this.getT('tip');
        }
        get transactionVersion() {
            return this.getT('transactionVersion');
        }
        get assetId() {
            return this.getT('assetId');
        }
        get metadataHash() {
            return this.getT('metadataHash');
        }
        sign(_signerPair) {
            throw new Error('Extrinsic: ExtrinsicV5 does not include signing support');
        }
    }

    class GenericExtrinsicSignatureV5 extends Struct {
        #signKeys;
        constructor(registry, value, { isSigned } = {}) {
            const signTypes = registry.getSignedExtensionTypes();
            super(registry, util.objectSpread(
            { signer: 'Address', signature: 'ExtrinsicSignature', transactionExtensionVersion: 'u8' }, signTypes), GenericExtrinsicSignatureV5.decodeExtrinsicSignature(value, isSigned));
            this.#signKeys = Object.keys(signTypes);
            util.objectProperties(this, this.#signKeys, (k) => this.get(k));
        }
        static decodeExtrinsicSignature(value, isSigned = false) {
            if (!value) {
                return EMPTY_U8A;
            }
            else if (value instanceof GenericExtrinsicSignatureV5) {
                return value;
            }
            return isSigned
                ? value
                : EMPTY_U8A;
        }
        get encodedLength() {
            return this.isSigned
                ? super.encodedLength
                : 0;
        }
        get isSigned() {
            return !this.signature.isEmpty;
        }
        get era() {
            return this.getT('era');
        }
        get nonce() {
            return this.getT('nonce');
        }
        get signature() {
            return (this.multiSignature.value || this.multiSignature);
        }
        get multiSignature() {
            return this.getT('signature');
        }
        get signer() {
            return this.getT('signer');
        }
        get tip() {
            return this.getT('tip');
        }
        get assetId() {
            return this.getT('assetId');
        }
        get mode() {
            return this.getT('mode');
        }
        get metadataHash() {
            return this.getT('metadataHash');
        }
        get transactionExtensionVersion() {
            return this.getT('transactionExtensionVersion');
        }
        _injectSignature(_signer, _signature, _payload) {
            throw new Error('Extrinsic: ExtrinsicV5 does not include signing support');
        }
        addSignature(_signer, _signature, _payload) {
            throw new Error('Extrinsic: ExtrinsicV5 does not include signing support');
        }
        createPayload(method, options) {
            const { era, runtimeVersion: { specVersion, transactionVersion } } = options;
            return new GenericExtrinsicPayloadV5(this.registry, util.objectSpread({}, options, {
                era: era || IMMORTAL_ERA,
                method: method.toHex(),
                specVersion,
                transactionVersion
            }));
        }
        sign(_method, _account, _options) {
            throw new Error('Extrinsic: ExtrinsicV5 does not include signing support');
        }
        signFake(_method, _address, _options) {
            throw new Error('Extrinsic: ExtrinsicV5 does not include signing support');
        }
        toU8a(isBare) {
            return this.isSigned
                ? super.toU8a(isBare)
                : EMPTY_U8A;
        }
    }

    function decodeU8a$4(u8a) {
        if (!u8a.length) {
            return new Uint8Array();
        }
        const [offset, length] = util.compactFromU8a(u8a);
        const total = offset + length.toNumber();
        if (total > u8a.length) {
            throw new Error(`Extrinsic: length less than remainder, expected at least ${total}, found ${u8a.length}`);
        }
        const data = u8a.subarray(offset, total);
        if (data[0] !== 69) {
            throw new Error(`Extrinsic: incorrect version for General Transactions, expected 5, found ${data[0] & UNMASK_VERSION}`);
        }
        return data.subarray(1);
    }
    class GeneralExtrinsic extends Struct {
        #version;
        #preamble;
        constructor(registry, value, opt) {
            const extTypes = registry.getSignedExtensionTypes();
            super(registry, util.objectSpread({
                transactionExtensionVersion: 'u8'
            }, extTypes, {
                method: 'Call'
            }), GeneralExtrinsic.decodeExtrinsic(registry, value));
            this.#version = opt?.version || 0b00000101;
            this.#preamble = 0b01000000;
        }
        static decodeExtrinsic(registry, value) {
            if (!value) {
                return EMPTY_U8A;
            }
            else if (value instanceof GeneralExtrinsic) {
                return value;
            }
            else if (util.isU8a(value) || Array.isArray(value) || util.isHex(value)) {
                return decodeU8a$4(util.u8aToU8a(value));
            }
            else if (util.isObject(value)) {
                const { payload, transactionExtensionVersion } = value;
                return util.objectSpread(payload || {}, {
                    transactionExtensionVersion: transactionExtensionVersion || registry.getTransactionExtensionVersion()
                });
            }
            return {};
        }
        get encodedLength() {
            return super.encodedLength;
        }
        get era() {
            return this.getT('era');
        }
        get nonce() {
            return this.getT('nonce');
        }
        get tip() {
            return this.getT('tip');
        }
        get assetId() {
            return this.getT('assetId');
        }
        get mode() {
            return this.getT('mode');
        }
        get metadataHash() {
            return this.getT('metadataHash');
        }
        get transactionExtensionVersion() {
            return this.getT('transactionExtensionVersion');
        }
        get method() {
            return this.getT('method');
        }
        get version() {
            return this.#version;
        }
        get preamble() {
            return this.#preamble;
        }
        toHex(isBare) {
            return util.u8aToHex(this.toU8a(isBare));
        }
        toU8a(isBare) {
            return isBare
                ? this.encode()
                : util.compactAddLength(this.encode());
        }
        toRawType() {
            return 'GeneralExt';
        }
        encode() {
            return util.u8aConcat(new Uint8Array([this.version | this.preamble]), super.toU8a());
        }
        signFake() {
            throw new Error('Extrinsic: Type GeneralExtrinsic does not have signFake implemented');
        }
        addSignature() {
            throw new Error('Extrinsic: Type GeneralExtrinsic does not have addSignature implemented');
        }
        sign() {
            throw new Error('Extrinsic: Type GeneralExtrinsic does not have sign implemented');
        }
        signature() {
            throw new Error('Extrinsic: Type GeneralExtrinsic does not have the signature getter');
        }
    }

    function decodeAccountId$1(value) {
        if (util.isU8a(value) || Array.isArray(value)) {
            return util.u8aToU8a(value);
        }
        else if (!value) {
            return new Uint8Array();
        }
        else if (util.isHex(value)) {
            return util.hexToU8a(value);
        }
        else if (util.isString(value)) {
            return utilCrypto.decodeAddress(value.toString());
        }
        throw new Error(`Unknown type passed to AccountId constructor, found typeof ${typeof value}`);
    }
    class BaseAccountId extends U8aFixed {
        constructor(registry, allowedBits = 256 | 264, value) {
            const decoded = decodeAccountId$1(value);
            const decodedBits = decoded.length * 8;
            if (decodedBits < allowedBits && decoded.some((b) => b)) {
                throw new Error(`Invalid AccountId provided, expected ${allowedBits >> 3} bytes, found ${decoded.length}`);
            }
            super(registry, decoded, allowedBits);
        }
        eq(other) {
            return super.eq(decodeAccountId$1(other));
        }
        toHuman() {
            return this.toJSON();
        }
        toJSON() {
            return this.toString();
        }
        toPrimitive() {
            return this.toJSON();
        }
        toString() {
            return utilCrypto.encodeAddress(this, this.registry.chainSS58);
        }
        toRawType() {
            return 'AccountId';
        }
    }
    class GenericAccountId extends BaseAccountId {
        constructor(registry, value) {
            super(registry, 256, value);
        }
    }
    class GenericAccountId33 extends BaseAccountId {
        constructor(registry, value) {
            super(registry, 264, value);
        }
    }

    const PREFIX_1BYTE = 0xef;
    const PREFIX_2BYTE = 0xfc;
    const PREFIX_4BYTE = 0xfd;
    const PREFIX_8BYTE = 0xfe;
    const MAX_1BYTE = new util.BN(PREFIX_1BYTE);
    const MAX_2BYTE = new util.BN(1).shln(16);
    const MAX_4BYTE = new util.BN(1).shln(32);
    function decodeAccountIndex(value) {
        if (value instanceof GenericAccountIndex) {
            return value.toBn();
        }
        else if (util.isBn(value) || util.isNumber(value) || util.isHex(value) || util.isU8a(value) || util.isBigInt(value)) {
            return value;
        }
        return decodeAccountIndex(utilCrypto.decodeAddress(value));
    }
    class GenericAccountIndex extends u32 {
        constructor(registry, value = new util.BN(0)) {
            super(registry, decodeAccountIndex(value));
        }
        static calcLength(_value) {
            const value = util.bnToBn(_value);
            if (value.lte(MAX_1BYTE)) {
                return 1;
            }
            else if (value.lt(MAX_2BYTE)) {
                return 2;
            }
            else if (value.lt(MAX_4BYTE)) {
                return 4;
            }
            return 8;
        }
        static readLength(input) {
            const first = input[0];
            if (first === PREFIX_2BYTE) {
                return [1, 2];
            }
            else if (first === PREFIX_4BYTE) {
                return [1, 4];
            }
            else if (first === PREFIX_8BYTE) {
                return [1, 8];
            }
            return [0, 1];
        }
        static writeLength(input) {
            switch (input.length) {
                case 2: return new Uint8Array([PREFIX_2BYTE]);
                case 4: return new Uint8Array([PREFIX_4BYTE]);
                case 8: return new Uint8Array([PREFIX_8BYTE]);
                default: return new Uint8Array([]);
            }
        }
        eq(other) {
            if (util.isBn(other) || util.isNumber(other)) {
                return super.eq(other);
            }
            return super.eq(this.registry.createTypeUnsafe('AccountIndex', [other]));
        }
        toHuman() {
            return this.toJSON();
        }
        toJSON() {
            return this.toString();
        }
        toPrimitive() {
            return this.toJSON();
        }
        toString() {
            const length = GenericAccountIndex.calcLength(this);
            return utilCrypto.encodeAddress(this.toU8a().subarray(0, length), this.registry.chainSS58);
        }
        toRawType() {
            return 'AccountIndex';
        }
    }

    class GenericBlock extends Struct {
        constructor(registry, value) {
            super(registry, {
                header: 'Header',
                extrinsics: 'Vec<Extrinsic>'
            }, value);
        }
        get contentHash() {
            return this.registry.hash(this.toU8a());
        }
        get extrinsics() {
            return this.getT('extrinsics');
        }
        get hash() {
            return this.header.hash;
        }
        get header() {
            return this.getT('header');
        }
    }

    function getArgsDef(registry, meta) {
        return meta.fields.reduce((result, { name, type }, index) => {
            result[name.unwrapOr(`param${index}`).toString()] = registry.createLookupType(type);
            return result;
        }, {});
    }
    function decodeCallViaObject(registry, value, _meta) {
        const { args, callIndex } = value;
        const lookupIndex = callIndex instanceof GenericCallIndex
            ? callIndex.toU8a()
            : callIndex;
        const meta = _meta || registry.findMetaCall(lookupIndex).meta;
        return {
            args,
            argsDef: getArgsDef(registry, meta),
            callIndex,
            meta
        };
    }
    function decodeCallViaU8a(registry, value, _meta) {
        const callIndex = registry.firstCallIndex.slice();
        callIndex.set(value.subarray(0, 2), 0);
        const meta = _meta || registry.findMetaCall(callIndex).meta;
        return {
            args: value.subarray(2),
            argsDef: getArgsDef(registry, meta),
            callIndex,
            meta
        };
    }
    function decodeCall(registry, value = new Uint8Array(), _meta) {
        if (util.isU8a(value) || util.isHex(value)) {
            return decodeCallViaU8a(registry, util.u8aToU8a(value), _meta);
        }
        else if (util.isObject(value) && value.callIndex && value.args) {
            return decodeCallViaObject(registry, value, _meta);
        }
        throw new Error(`Call: Cannot decode value '${value}' of type ${typeof value}`);
    }
    class GenericCallIndex extends U8aFixed {
        constructor(registry, value) {
            super(registry, value, 16);
        }
        toPrimitive() {
            return this.toHex();
        }
    }
    class GenericCall extends Struct {
        _meta;
        constructor(registry, value, meta) {
            const decoded = decodeCall(registry, value, meta);
            try {
                super(registry, {
                    callIndex: GenericCallIndex,
                    args: Struct.with(decoded.argsDef)
                }, decoded);
            }
            catch (error) {
                let method = 'unknown.unknown';
                try {
                    const c = registry.findMetaCall(decoded.callIndex);
                    method = `${c.section}.${c.method}`;
                }
                catch {
                }
                throw new Error(`Call: failed decoding ${method}:: ${error.message}`);
            }
            this._meta = decoded.meta;
        }
        get args() {
            return [...this.getT('args').values()];
        }
        get argsDef() {
            return getArgsDef(this.registry, this.meta);
        }
        get argsEntries() {
            return [...this.getT('args').entries()];
        }
        get callIndex() {
            return this.getT('callIndex').toU8a();
        }
        get data() {
            return this.getT('args').toU8a();
        }
        get meta() {
            return this._meta;
        }
        get method() {
            return this.registry.findMetaCall(this.callIndex).method;
        }
        get section() {
            return this.registry.findMetaCall(this.callIndex).section;
        }
        is(other) {
            return other.callIndex[0] === this.callIndex[0] && other.callIndex[1] === this.callIndex[1];
        }
        toHuman(isExpanded, disableAscii) {
            let call;
            try {
                call = this.registry.findMetaCall(this.callIndex);
            }
            catch {
            }
            return util.objectSpread({
                args: this.argsEntries.reduce((args, [n, a]) => util.objectSpread(args, { [n]: a.toHuman(isExpanded, disableAscii) }), {}),
                method: call?.method,
                section: call?.section
            }, isExpanded && call
                ? { docs: call.meta.docs.map((d) => d.toString()) }
                : null);
        }
        toRawType() {
            return 'Call';
        }
    }

    function createValue(registry, type, value, asArray = true) {
        if (value && util.isFunction(value.unwrapOrDefault)) {
            return value;
        }
        return registry.createTypeUnsafe(type, [
            asArray
                ? util.isNull(value) || util.isUndefined(value)
                    ? null
                    : Array.isArray(value)
                        ? value
                        : [value]
                : value
        ]);
    }
    function decodeValue(registry, key, value) {
        return key === 'ss58Format'
            ? createValue(registry, 'Option<u32>', value, false)
            : key === 'tokenDecimals'
                ? createValue(registry, 'Option<Vec<u32>>', value)
                : key === 'tokenSymbol'
                    ? createValue(registry, 'Option<Vec<Text>>', value)
                    : key === 'isEthereum'
                        ? createValue(registry, 'Bool', value, false)
                        : value;
    }
    function decode(registry, value) {
        return (
        value && util.isFunction(value.entries)
            ? [...value.entries()]
            : Object.entries(value || {})).reduce((all, [key, value]) => {
            all[key] = decodeValue(registry, key, value);
            return all;
        }, {
            isEthereum: registry.createTypeUnsafe('Bool', []),
            ss58Format: registry.createTypeUnsafe('Option<u32>', []),
            tokenDecimals: registry.createTypeUnsafe('Option<Vec<u32>>', []),
            tokenSymbol: registry.createTypeUnsafe('Option<Vec<Text>>', [])
        });
    }
    class GenericChainProperties extends Json {
        constructor(registry, value) {
            super(registry, decode(registry, value));
        }
        get isEthereum() {
            return this.getT('isEthereum');
        }
        get ss58Format() {
            return this.getT('ss58Format');
        }
        get tokenDecimals() {
            return this.getT('tokenDecimals');
        }
        get tokenSymbol() {
            return this.getT('tokenSymbol');
        }
    }

    const CID_AURA =  util.stringToU8a('aura');
    const CID_BABE =  util.stringToU8a('BABE');
    const CID_GRPA =  util.stringToU8a('FRNK');
    const CID_POW =  util.stringToU8a('pow_');
    const CID_NMBS =  util.stringToU8a('nmbs');
    function getAuraAuthor(registry, bytes, sessionValidators) {
        return sessionValidators[registry.createTypeUnsafe('RawAuraPreDigest', [bytes.toU8a(true)])
            .slotNumber
            .mod(new util.BN(sessionValidators.length))
            .toNumber()];
    }
    function getBabeAuthor(registry, bytes, sessionValidators) {
        const digest = registry.createTypeUnsafe('RawBabePreDigestCompat', [bytes.toU8a(true)]);
        return sessionValidators[digest.value.toNumber()];
    }
    function getBytesAsAuthor(registry, bytes) {
        return registry.createTypeUnsafe('AccountId', [bytes]);
    }
    class GenericConsensusEngineId extends U8aFixed {
        constructor(registry, value) {
            super(registry, util.isNumber(value)
                ? util.bnToU8a(value, { isLe: false })
                : value, 32);
        }
        get isAura() {
            return this.eq(CID_AURA);
        }
        get isBabe() {
            return this.eq(CID_BABE);
        }
        get isGrandpa() {
            return this.eq(CID_GRPA);
        }
        get isPow() {
            return this.eq(CID_POW);
        }
        get isNimbus() {
            return this.eq(CID_NMBS);
        }
        extractAuthor(bytes, sessionValidators) {
            if (sessionValidators?.length) {
                if (this.isAura) {
                    return getAuraAuthor(this.registry, bytes, sessionValidators);
                }
                else if (this.isBabe) {
                    return getBabeAuthor(this.registry, bytes, sessionValidators);
                }
            }
            if (this.isPow || this.isNimbus) {
                return getBytesAsAuthor(this.registry, bytes);
            }
            return undefined;
        }
        toHuman() {
            return this.toString();
        }
        toRawType() {
            return 'ConsensusEngineId';
        }
        toString() {
            return this.isAscii
                ? util.u8aToString(this)
                : util.u8aToHex(this);
        }
    }

    const ACCOUNT_ID_PREFIX$1 = new Uint8Array([0xff]);
    function decodeString$1(registry, value) {
        const decoded = utilCrypto.decodeAddress(value);
        return decoded.length === 32
            ? registry.createTypeUnsafe('AccountId', [decoded])
            : registry.createTypeUnsafe('AccountIndex', [util.u8aToBn(decoded)]);
    }
    function decodeU8a$3(registry, value) {
        if (value.length === 32) {
            return registry.createTypeUnsafe('AccountId', [value]);
        }
        else if (value[0] === 0xff) {
            return registry.createTypeUnsafe('AccountId', [value.subarray(1)]);
        }
        const [offset, length] = GenericAccountIndex.readLength(value);
        return registry.createTypeUnsafe('AccountIndex', [util.u8aToBn(value.subarray(offset, offset + length))]);
    }
    function decodeAddressOrIndex$1(registry, value) {
        return value instanceof GenericLookupSource
            ? value.inner
            : value instanceof GenericAccountId || value instanceof GenericAccountIndex
                ? value
                : util.isBn(value) || util.isNumber(value) || util.isBigInt(value)
                    ? registry.createTypeUnsafe('AccountIndex', [value])
                    : Array.isArray(value) || util.isHex(value) || util.isU8a(value)
                        ? decodeU8a$3(registry, util.u8aToU8a(value))
                        : decodeString$1(registry, value);
    }
    class GenericLookupSource extends AbstractBase {
        constructor(registry, value = new Uint8Array()) {
            super(registry, decodeAddressOrIndex$1(registry, value));
        }
        get encodedLength() {
            const rawLength = this._rawLength;
            return rawLength + (
            rawLength > 1
                ? 1
                : 0);
        }
        get _rawLength() {
            return this.inner instanceof GenericAccountIndex
                ? GenericAccountIndex.calcLength(this.inner)
                : this.inner.encodedLength;
        }
        inspect() {
            const value = this.inner.toU8a().subarray(0, this._rawLength);
            return {
                outer: [
                    new Uint8Array(this.inner instanceof GenericAccountIndex
                        ? GenericAccountIndex.writeLength(value)
                        : ACCOUNT_ID_PREFIX$1),
                    value
                ]
            };
        }
        toHex() {
            return util.u8aToHex(this.toU8a());
        }
        toRawType() {
            return 'Address';
        }
        toU8a(isBare) {
            const encoded = this.inner.toU8a().subarray(0, this._rawLength);
            return isBare
                ? encoded
                : util.u8aConcat(this.inner instanceof GenericAccountIndex
                    ? GenericAccountIndex.writeLength(encoded)
                    : ACCOUNT_ID_PREFIX$1, encoded);
        }
    }

    function decodeU8a$2(registry, u8a) {
        if ([0, 32].includes(u8a.length)) {
            return { Id: u8a };
        }
        else if (u8a.length === 20) {
            return { Address20: u8a };
        }
        else if (u8a.length <= 8) {
            return { Index: registry.createTypeUnsafe('AccountIndex', [u8a]).toNumber() };
        }
        return u8a;
    }
    function decodeMultiAny(registry, value) {
        if (value instanceof GenericAccountId) {
            return { Id: value };
        }
        else if (util.isU8a(value)) {
            return decodeU8a$2(registry, value);
        }
        else if (value instanceof GenericMultiAddress) {
            return value;
        }
        else if (value instanceof GenericAccountIndex || util.isBn(value) || util.isNumber(value)) {
            return { Index: util.isNumber(value) ? value : value.toNumber() };
        }
        else if (util.isString(value)) {
            return decodeU8a$2(registry, utilCrypto.decodeAddress(value.toString()));
        }
        return value;
    }
    class GenericMultiAddress extends Enum {
        constructor(registry, value) {
            super(registry, {
                Id: 'AccountId',
                Index: 'Compact<AccountIndex>',
                Raw: 'Bytes',
                Address32: 'H256',
                Address20: 'H160'
            }, decodeMultiAny(registry, value));
        }
        inspect() {
            const { inner, outer = [] } = this.inner.inspect();
            return {
                inner,
                outer: [new Uint8Array([this.index]), ...outer]
            };
        }
        toString() {
            return this.value.toString();
        }
    }

    const AYE_BITS = 0b10000000;
    const NAY_BITS = 0b00000000;
    const CON_MASK = 0b01111111;
    const DEF_CONV = 0b00000000;
    function decodeVoteBool(value) {
        return value
            ? new Uint8Array([AYE_BITS | DEF_CONV])
            : new Uint8Array([NAY_BITS]);
    }
    function decodeVoteU8a(value) {
        return value.length
            ? value.subarray(0, 1)
            : new Uint8Array([NAY_BITS]);
    }
    function decodeVoteType(registry, value) {
        return new Uint8Array([
            (new bool(registry, value.aye).isTrue
                ? AYE_BITS
                : NAY_BITS) |
                registry.createTypeUnsafe('Conviction', [value.conviction || DEF_CONV]).index
        ]);
    }
    function decodeVote(registry, value) {
        if (util.isU8a(value)) {
            return decodeVoteU8a(value);
        }
        else if (util.isUndefined(value) || value instanceof Boolean || util.isBoolean(value)) {
            return decodeVoteBool(new bool(registry, value).isTrue);
        }
        else if (util.isNumber(value)) {
            return decodeVoteBool(value < 0);
        }
        return decodeVoteType(registry, value);
    }
    class GenericVote extends U8aFixed {
        #aye;
        #conviction;
        constructor(registry, value) {
            const decoded = decodeVote(registry, value);
            super(registry, decoded, 8);
            this.#aye = (decoded[0] & AYE_BITS) === AYE_BITS;
            this.#conviction = this.registry.createTypeUnsafe('Conviction', [decoded[0] & CON_MASK]);
        }
        get conviction() {
            return this.#conviction;
        }
        get isAye() {
            return this.#aye;
        }
        get isNay() {
            return !this.isAye;
        }
        toHuman(isExpanded) {
            return {
                conviction: this.conviction.toHuman(isExpanded),
                vote: this.isAye ? 'Aye' : 'Nay'
            };
        }
        toPrimitive() {
            return {
                aye: this.isAye,
                conviction: this.conviction.toPrimitive()
            };
        }
        toRawType() {
            return 'Vote';
        }
    }

    function decodeAccountId(value) {
        if (util.isU8a(value) || Array.isArray(value)) {
            return util.u8aToU8a(value);
        }
        else if (util.isHex(value) || utilCrypto.isEthereumAddress(value.toString())) {
            return util.hexToU8a(value.toString());
        }
        else if (util.isString(value)) {
            return util.u8aToU8a(value);
        }
        return value;
    }
    class GenericEthereumAccountId extends U8aFixed {
        constructor(registry, value = new Uint8Array()) {
            super(registry, decodeAccountId(value), 160);
        }
        eq(other) {
            return !!other && super.eq(decodeAccountId(other));
        }
        toHuman() {
            return this.toJSON();
        }
        toJSON() {
            return this.toString();
        }
        toPrimitive() {
            return this.toJSON();
        }
        toString() {
            return utilCrypto.ethereumEncode(this);
        }
        toRawType() {
            return 'AccountId';
        }
    }

    const ACCOUNT_ID_PREFIX = new Uint8Array([0xff]);
    function decodeString(registry, value) {
        const decoded = utilCrypto.decodeAddress(value);
        return decoded.length === 20
            ? registry.createTypeUnsafe('EthereumAccountId', [decoded])
            : registry.createTypeUnsafe('AccountIndex', [util.u8aToBn(decoded)]);
    }
    function decodeU8a$1(registry, value) {
        if (value.length === 20) {
            return registry.createTypeUnsafe('EthereumAccountId', [value]);
        }
        else if (value[0] === 0xff) {
            return registry.createTypeUnsafe('EthereumAccountId', [value.subarray(1)]);
        }
        const [offset, length] = GenericAccountIndex.readLength(value);
        return registry.createTypeUnsafe('AccountIndex', [util.u8aToBn(value.subarray(offset, offset + length))]);
    }
    function decodeAddressOrIndex(registry, value) {
        return value instanceof GenericEthereumLookupSource
            ? value.inner
            : value instanceof GenericEthereumAccountId || value instanceof GenericAccountIndex
                ? value
                : util.isU8a(value) || Array.isArray(value) || util.isHex(value)
                    ? decodeU8a$1(registry, util.u8aToU8a(value))
                    : util.isBn(value) || util.isNumber(value) || util.isBigInt(value)
                        ? registry.createTypeUnsafe('AccountIndex', [value])
                        : decodeString(registry, value);
    }
    class GenericEthereumLookupSource extends AbstractBase {
        constructor(registry, value = new Uint8Array()) {
            super(registry, decodeAddressOrIndex(registry, value));
        }
        get encodedLength() {
            const rawLength = this._rawLength;
            return rawLength + (
            rawLength > 1
                ? 1
                : 0);
        }
        get _rawLength() {
            return this.inner instanceof GenericAccountIndex
                ? GenericAccountIndex.calcLength(this.inner)
                : this.inner.encodedLength;
        }
        toHex() {
            return util.u8aToHex(this.toU8a());
        }
        toRawType() {
            return 'Address';
        }
        toU8a(isBare) {
            const encoded = this.inner.toU8a().subarray(0, this._rawLength);
            return isBare
                ? encoded
                : util.u8aConcat(this.inner instanceof GenericAccountIndex
                    ? GenericAccountIndex.writeLength(encoded)
                    : ACCOUNT_ID_PREFIX, encoded);
        }
    }

    function decodeDataU8a(registry, value) {
        const indicator = value[0];
        if (!indicator) {
            return [undefined, undefined];
        }
        else if (indicator >= 1 && indicator <= 33) {
            const length = indicator - 1;
            const data = value.subarray(1, length + 1);
            return [registry.createTypeUnsafe('Raw', [data]), 1];
        }
        else if (indicator >= 34 && indicator <= 37) {
            return [value.subarray(1, 32 + 1), indicator - 32];
        }
        throw new Error(`Unable to decode Data, invalid indicator byte ${indicator}`);
    }
    function decodeData(registry, value) {
        if (util.isU8a(value) || util.isString(value)) {
            return decodeDataU8a(registry, util.u8aToU8a(value));
        }
        else if (!value) {
            return [undefined, undefined];
        }
        return [value, undefined];
    }
    class Data extends Enum {
        constructor(registry, value) {
            super(registry, {
                None: 'Null',
                Raw: 'Bytes',
                BlakeTwo256: 'H256',
                Sha256: 'H256',
                Keccak256: 'H256',
                ShaThree256: 'H256'
            }, ...decodeData(registry, value));
            if (this.isRaw && this.asRaw.length > 32) {
                throw new Error('Data.Raw values are limited to a maximum length of 32 bytes');
            }
        }
        get asBlakeTwo256() {
            return this.value;
        }
        get asKeccak256() {
            return this.value;
        }
        get asRaw() {
            return this.value;
        }
        get asSha256() {
            return this.value;
        }
        get asShaThree256() {
            return this.value;
        }
        get isBlakeTwo256() {
            return this.index === 2;
        }
        get isKeccak256() {
            return this.index === 4;
        }
        get isNone() {
            return this.index === 0;
        }
        get isRaw() {
            return this.index === 1;
        }
        get isSha256() {
            return this.index === 3;
        }
        get isShaThree256() {
            return this.index === 5;
        }
        get encodedLength() {
            return this.toU8a().length;
        }
        toU8a() {
            if (this.index === 0) {
                return new Uint8Array(1);
            }
            else if (this.index === 1) {
                const data = this.value.toU8a(true);
                const length = Math.min(data.length, 32);
                const u8a = new Uint8Array(length + 1);
                u8a.set([length + 1], 0);
                u8a.set(data.subarray(0, length), 1);
                return u8a;
            }
            const u8a = new Uint8Array(33);
            u8a.set([this.index + 32], 0);
            u8a.set(this.value.toU8a(), 1);
            return u8a;
        }
    }

    const HASHER_MAP = {
        Blake2_128: [16, false],
        Blake2_128Concat: [16, true],
        Blake2_256: [32, false],
        Identity: [0, true],
        Twox128: [16, false],
        Twox256: [32, false],
        Twox64Concat: [8, true]
    };
    function decodeStorageKey(value) {
        if (util.isU8a(value) || !value || util.isString(value)) {
            return { key: value };
        }
        else if (value instanceof StorageKey) {
            return {
                key: value,
                method: value.method,
                section: value.section
            };
        }
        else if (util.isFunction(value)) {
            return {
                key: value(),
                method: value.method,
                section: value.section
            };
        }
        else if (Array.isArray(value)) {
            const [fn, args = []] = value;
            if (!util.isFunction(fn)) {
                throw new Error('Expected function input for key construction');
            }
            if (fn.meta && fn.meta.type.isMap) {
                const map = fn.meta.type.asMap;
                if (!Array.isArray(args) || args.length !== map.hashers.length) {
                    throw new Error(`Expected an array of ${map.hashers.length} values as params to a Map query`);
                }
            }
            return {
                key: fn(...args),
                method: fn.method,
                section: fn.section
            };
        }
        throw new Error(`Unable to convert input ${value} to StorageKey`);
    }
    function decodeHashers(registry, value, hashers) {
        let offset = 32;
        const count = hashers.length;
        const result = new Array(count);
        for (let i = 0; i < count; i++) {
            const [hasher, type] = hashers[i];
            const [hashLen, canDecode] = HASHER_MAP[hasher.type];
            const decoded = canDecode
                ? registry.createTypeUnsafe(getSiName(registry.lookup, type), [value.subarray(offset + hashLen)])
                : registry.createTypeUnsafe('Raw', [value.subarray(offset, offset + hashLen)]);
            offset += hashLen + (canDecode ? decoded.encodedLength : 0);
            result[i] = decoded;
        }
        return result;
    }
    function decodeArgsFromMeta(registry, value, meta) {
        if (!meta || !meta.type.isMap) {
            return [];
        }
        const { hashers, key } = meta.type.asMap;
        const keys = hashers.length === 1
            ? [key]
            : registry.lookup.getSiType(key).def.asTuple;
        return decodeHashers(registry, value, hashers.map((h, i) => [h, keys[i]]));
    }
    function getMeta(value) {
        if (value instanceof StorageKey) {
            return value.meta;
        }
        else if (util.isFunction(value)) {
            return value.meta;
        }
        else if (Array.isArray(value)) {
            const [fn] = value;
            return fn.meta;
        }
        return undefined;
    }
    function getType(registry, value) {
        if (value instanceof StorageKey) {
            return value.outputType;
        }
        else if (util.isFunction(value)) {
            return unwrapStorageType(registry, value.meta.type);
        }
        else if (Array.isArray(value)) {
            const [fn] = value;
            if (fn.meta) {
                return unwrapStorageType(registry, fn.meta.type);
            }
        }
        return 'Raw';
    }
    class StorageKey extends Bytes {
        #args;
        #meta;
        #outputType;
        #method;
        #section;
        constructor(registry, value, override = {}) {
            const { key, method, section } = decodeStorageKey(value);
            super(registry, key);
            this.#outputType = getType(registry, value);
            this.setMeta(getMeta(value), override.section || section, override.method || method);
        }
        get args() {
            return this.#args;
        }
        get meta() {
            return this.#meta;
        }
        get method() {
            return this.#method;
        }
        get outputType() {
            return this.#outputType;
        }
        get section() {
            return this.#section;
        }
        is(key) {
            return key.section === this.section && key.method === this.method;
        }
        setMeta(meta, section, method) {
            this.#meta = meta;
            this.#method = method || this.#method;
            this.#section = section || this.#section;
            if (meta) {
                this.#outputType = unwrapStorageType(this.registry, meta.type);
            }
            try {
                this.#args = decodeArgsFromMeta(this.registry, this.toU8a(true), meta);
            }
            catch {
            }
            return this;
        }
        toHuman(_isExtended, disableAscii) {
            return this.#args.length
                ? this.#args.map((a) => a.toHuman(undefined, disableAscii))
                : super.toHuman(undefined, disableAscii);
        }
        toRawType() {
            return 'StorageKey';
        }
    }

    const baseTypes = /*#__PURE__*/Object.freeze({
        __proto__: null,
        BitVec: BitVec,
        Bool: bool,
        Bytes: Bytes,
        Data: Data,
        F32: f32,
        F64: f64,
        GeneralExtrinsic: GeneralExtrinsic,
        GenericAccountId: GenericAccountId,
        GenericAccountId32: GenericAccountId,
        GenericAccountId33: GenericAccountId33,
        GenericAccountIndex: GenericAccountIndex,
        GenericAddress: GenericMultiAddress,
        GenericBlock: GenericBlock,
        GenericCall: GenericCall,
        GenericChainProperties: GenericChainProperties,
        GenericConsensusEngineId: GenericConsensusEngineId,
        GenericEthereumAccountId: GenericEthereumAccountId,
        GenericEthereumLookupSource: GenericEthereumLookupSource,
        GenericEvent: GenericEvent,
        GenericEventData: GenericEventData,
        GenericExtrinsic: GenericExtrinsic,
        GenericExtrinsicEra: GenericExtrinsicEra,
        GenericExtrinsicPayload: GenericExtrinsicPayload,
        GenericExtrinsicPayloadUnknown: GenericExtrinsicPayloadUnknown,
        GenericExtrinsicPayloadV4: GenericExtrinsicPayloadV4,
        GenericExtrinsicPayloadV5: GenericExtrinsicPayloadV5,
        GenericExtrinsicSignatureV4: GenericExtrinsicSignatureV4,
        GenericExtrinsicSignatureV5: GenericExtrinsicSignatureV5,
        GenericExtrinsicUnknown: GenericExtrinsicUnknown,
        GenericExtrinsicV4: GenericExtrinsicV4,
        GenericExtrinsicV5: GenericExtrinsicV5,
        GenericImmortalEra: ImmortalEra,
        GenericLookupSource: GenericLookupSource,
        GenericMortalEra: MortalEra,
        GenericMultiAddress: GenericMultiAddress,
        GenericSignerPayload: GenericSignerPayload,
        GenericVote: GenericVote,
        I128: i128,
        I16: i16,
        I256: i256,
        I32: i32,
        I64: i64,
        I8: i8,
        ISize: isize,
        Null: Null,
        OptionBool: OptionBool,
        StorageKey: StorageKey,
        Text: Text,
        Type: Type,
        U128: u128,
        U16: u16,
        U256: u256,
        U32: u32,
        U64: u64,
        U8: u8,
        USize: usize,
        bool: bool,
        f32: f32,
        f64: f64,
        i128: i128,
        i16: i16,
        i256: i256,
        i32: i32,
        i64: i64,
        i8: i8,
        isize: isize,
        u128: u128,
        u16: u16,
        u256: u256,
        u32: u32,
        u64: u64,
        u8: u8,
        usize: usize
    });

    function convert(fn) {
        return ({ name }) => fn(name);
    }
    const objectNameToCamel =  convert(util.stringCamelCase);
    const objectNameToString =  convert((n) => n.toString());

    function isTx(tx, callIndex) {
        return tx.callIndex[0] === callIndex[0] && tx.callIndex[1] === callIndex[1];
    }
    function createUnchecked(registry, section, callIndex, callMetadata) {
        const expectedArgs = callMetadata.fields;
        const funcName = util.stringCamelCase(callMetadata.name);
        const extrinsicFn = (...args) => {
            if (expectedArgs.length !== args.length) {
                throw new Error(`Extrinsic ${section}.${funcName} expects ${expectedArgs.length} arguments, got ${args.length}.`);
            }
            return registry.createTypeUnsafe('Call', [{ args, callIndex }, callMetadata]);
        };
        extrinsicFn.is = (tx) => isTx(tx, callIndex);
        extrinsicFn.callIndex = callIndex;
        extrinsicFn.meta = callMetadata;
        extrinsicFn.method = funcName;
        extrinsicFn.section = section;
        extrinsicFn.toJSON = () => callMetadata.toJSON();
        return extrinsicFn;
    }

    function filterCallsSome({ calls }) {
        return calls.isSome;
    }
    function createCallFunction(registry, lookup, variant, sectionName, sectionIndex) {
        const { fields, index } = variant;
        const count = fields.length;
        const args = new Array(count);
        for (let i = 0; i < count; i++) {
            const { name, type, typeName } = fields[i];
            args[i] = util.objectSpread({
                name: util.stringCamelCase(name.unwrapOr(`param${i}`)),
                type: getSiName(lookup, type)
            }, typeName.isSome
                ? { typeName: typeName.unwrap() }
                : null);
        }
        return createUnchecked(registry, sectionName, new Uint8Array([sectionIndex, index.toNumber()]), registry.createTypeUnsafe('FunctionMetadataLatest', [util.objectSpread({ args }, variant)]));
    }
    function decorateExtrinsics(registry, { lookup, pallets }, version) {
        const result = {};
        const filtered = pallets.filter(filterCallsSome);
        for (let i = 0, count = filtered.length; i < count; i++) {
            const { calls, index, name } = filtered[i];
            const sectionName = util.stringCamelCase(name);
            const sectionIndex = version >= 12 ? index.toNumber() : i;
            util.lazyMethod(result, sectionName, () => lazyVariants(lookup, calls.unwrap(), objectNameToCamel, (variant) => createCallFunction(registry, lookup, variant, sectionName, sectionIndex)));
        }
        return result;
    }

    function createStorageHasher(registry, hasher) {
        if (hasher.toNumber() >= 2) {
            return registry.createTypeUnsafe('StorageHasherV10', [hasher.toNumber() + 1]);
        }
        return registry.createTypeUnsafe('StorageHasherV10', [hasher]);
    }
    function createStorageType(registry, entryType) {
        if (entryType.isMap) {
            return [util.objectSpread({}, entryType.asMap, {
                    hasher: createStorageHasher(registry, entryType.asMap.hasher)
                }), 1];
        }
        if (entryType.isDoubleMap) {
            return [util.objectSpread({}, entryType.asDoubleMap, {
                    hasher: createStorageHasher(registry, entryType.asDoubleMap.hasher),
                    key2Hasher: createStorageHasher(registry, entryType.asDoubleMap.key2Hasher)
                }), 2];
        }
        return [entryType.asPlain, 0];
    }
    function convertModule(registry, mod) {
        const storage = mod.storage.unwrapOr(null);
        return registry.createTypeUnsafe('ModuleMetadataV10', [util.objectSpread({}, mod, {
                storage: storage
                    ? util.objectSpread({}, storage, {
                        items: storage.items.map((item) => util.objectSpread({}, item, {
                            type: registry.createTypeUnsafe('StorageEntryTypeV10', createStorageType(registry, item.type))
                        }))
                    })
                    : null
            })]);
    }
    function toV10(registry, { modules }) {
        return registry.createTypeUnsafe('MetadataV10', [{
                modules: modules.map((mod) => convertModule(registry, mod))
            }]);
    }

    function toV11(registry, { modules }) {
        return registry.createTypeUnsafe('MetadataV11', [{
                extrinsic: {
                    signedExtensions: [],
                    version: 0
                },
                modules
            }]);
    }

    function toV12(registry, { extrinsic, modules }) {
        return registry.createTypeUnsafe('MetadataV12', [{
                extrinsic,
                modules: modules.map((mod) => registry.createTypeUnsafe('ModuleMetadataV12', [util.objectSpread({}, mod, { index: 255 })]))
            }]);
    }

    function toV13(registry, v12) {
        return registry.createTypeUnsafe('MetadataV13', [v12]);
    }

    const typesAlias = {
        assets: {
            Approval: 'AssetApproval',
            ApprovalKey: 'AssetApprovalKey',
            Balance: 'TAssetBalance',
            DestroyWitness: 'AssetDestroyWitness'
        },
        babe: {
            EquivocationProof: 'BabeEquivocationProof'
        },
        balances: {
            Status: 'BalanceStatus'
        },
        beefy: {
            AuthorityId: 'BeefyId'
        },
        contracts: {
            StorageKey: 'ContractStorageKey'
        },
        electionProviderMultiPhase: {
            Phase: 'ElectionPhase'
        },
        ethereum: {
            Block: 'EthBlock',
            Header: 'EthHeader',
            Receipt: 'EthReceipt',
            Transaction: 'EthTransaction',
            TransactionStatus: 'EthTransactionStatus'
        },
        evm: {
            Account: 'EvmAccount',
            Log: 'EvmLog',
            Vicinity: 'EvmVicinity'
        },
        grandpa: {
            Equivocation: 'GrandpaEquivocation',
            EquivocationProof: 'GrandpaEquivocationProof'
        },
        identity: {
            Judgement: 'IdentityJudgement'
        },
        inclusion: {
            ValidatorIndex: 'ParaValidatorIndex'
        },
        paraDisputes: {
            ValidatorIndex: 'ParaValidatorIndex'
        },
        paraInclusion: {
            ValidatorIndex: 'ParaValidatorIndex'
        },
        paraScheduler: {
            ValidatorIndex: 'ParaValidatorIndex'
        },
        paraShared: {
            ValidatorIndex: 'ParaValidatorIndex'
        },
        parachains: {
            Id: 'ParaId'
        },
        parasDisputes: {
            ValidatorIndex: 'ParaValidatorIndex'
        },
        parasInclusion: {
            ValidatorIndex: 'ParaValidatorIndex'
        },
        parasScheduler: {
            ValidatorIndex: 'ParaValidatorIndex'
        },
        parasShared: {
            ValidatorIndex: 'ParaValidatorIndex'
        },
        proposeParachain: {
            Proposal: 'ParachainProposal'
        },
        proxy: {
            Announcement: 'ProxyAnnouncement'
        },
        scheduler: {
            ValidatorIndex: 'ParaValidatorIndex'
        },
        shared: {
            ValidatorIndex: 'ParaValidatorIndex'
        },
        society: {
            Judgement: 'SocietyJudgement',
            Vote: 'SocietyVote'
        },
        staking: {
            Compact: 'CompactAssignments'
        },
        treasury: {
            Proposal: 'TreasuryProposal'
        },
        xcm: {
            AssetId: 'XcmAssetId'
        },
        xcmPallet: {
            AssetId: 'XcmAssetId'
        }
    };
    function getAliasTypes({ knownTypes }, section) {
        return {
            ...(typesAlias[section] ?? {}),
            ...(knownTypes.typesAlias?.[section] ?? {})
        };
    }

    const BOXES = [['<', '>'], ['<', ','], [',', '>'], ['(', ')'], ['(', ','], [',', ','], [',', ')']];
    function compatType(specs, _type) {
        const type = _type.toString();
        const index = specs.findIndex(({ def }) => def.HistoricMetaCompat === type);
        if (index !== -1) {
            return index;
        }
        return specs.push({
            def: {
                HistoricMetaCompat: type
            }
        }) - 1;
    }
    function compatTypes(specs, ...types) {
        for (let i = 0, count = types.length; i < count; i++) {
            compatType(specs, types[i]);
        }
    }
    function makeTupleType(specs, entries) {
        return specs.push({
            def: {
                Tuple: entries
            }
        }) - 1;
    }
    function makeVariantType(modName, variantType, specs, variants) {
        return specs.push({
            def: {
                Variant: { variants }
            },
            path: [`pallet_${modName.toString()}`, 'pallet', variantType]
        }) - 1;
    }
    function registerOriginCaller(registry, modules, metaVersion) {
        registry.register({
            OriginCaller: {
                _enum: modules
                    .map((mod, index) => [
                    mod.name.toString(),
                    metaVersion >= 12
                        ? mod.index.toNumber()
                        : index
                ])
                    .sort((a, b) => a[1] - b[1])
                    .reduce((result, [name, index]) => {
                    for (let i = Object.keys(result).length; i < index; i++) {
                        result[`Empty${i}`] = 'Null';
                    }
                    result[name] = knownOrigins[name] || 'Null';
                    return result;
                }, {})
            }
        });
    }
    function setTypeOverride(sectionTypes, types) {
        types.forEach((type) => {
            const override = Object.keys(sectionTypes).find((aliased) => type.eq(aliased));
            if (override) {
                type.setOverride(sectionTypes[override]);
            }
            else {
                const orig = type.toString();
                const alias = Object
                    .entries(sectionTypes)
                    .reduce((result, [src, dst]) => BOXES.reduce((result, [a, z]) => result.replace(`${a}${src}${z}`, `${a}${dst}${z}`), result), orig);
                if (orig !== alias) {
                    type.setOverride(alias);
                }
            }
        });
    }
    function convertCalls$1(specs, registry, modName, calls, sectionTypes) {
        const variants = calls.map(({ args, docs, name }, index) => {
            setTypeOverride(sectionTypes, args.map(({ type }) => type));
            return registry.createTypeUnsafe('SiVariant', [{
                    docs,
                    fields: args.map(({ name, type }) => registry.createTypeUnsafe('SiField', [{ name, type: compatType(specs, type) }])),
                    index,
                    name
                }]);
        });
        return registry.createTypeUnsafe('PalletCallMetadataV14', [{
                type: makeVariantType(modName, 'Call', specs, variants)
            }]);
    }
    function convertConstants$1(specs, registry, constants, sectionTypes) {
        return constants.map(({ docs, name, type, value }) => {
            setTypeOverride(sectionTypes, [type]);
            return registry.createTypeUnsafe('PalletConstantMetadataV14', [{
                    docs,
                    name,
                    type: compatType(specs, type),
                    value
                }]);
        });
    }
    function convertErrors(specs, registry, modName, errors, _sectionTypes) {
        const variants = errors.map(({ docs, name }, index) => registry.createTypeUnsafe('SiVariant', [{
                docs,
                fields: [],
                index,
                name
            }]));
        return registry.createTypeUnsafe('PalletErrorMetadataV14', [{
                type: makeVariantType(modName, 'Error', specs, variants)
            }]);
    }
    function convertEvents$1(specs, registry, modName, events, sectionTypes) {
        const variants = events.map(({ args, docs, name }, index) => {
            setTypeOverride(sectionTypes, args);
            return registry.createTypeUnsafe('SiVariant', [{
                    docs,
                    fields: args.map((t) => registry.createTypeUnsafe('SiField', [{ type: compatType(specs, t) }])),
                    index,
                    name
                }]);
        });
        return registry.createTypeUnsafe('PalletEventMetadataV14', [{
                type: makeVariantType(modName, 'Event', specs, variants)
            }]);
    }
    function createMapEntry(specs, registry, sectionTypes, { hashers, isLinked, isOptional, keys, value }) {
        setTypeOverride(sectionTypes, [value, ...(Array.isArray(keys) ? keys : [keys])]);
        return registry.createTypeUnsafe('StorageEntryTypeV14', [{
                Map: {
                    hashers,
                    key: hashers.length === 1
                        ? compatType(specs, keys[0])
                        : makeTupleType(specs, keys.map((t) => compatType(specs, t))),
                    value: isLinked
                        ? compatType(specs, `(${isOptional ? `Option<${value.toString()}>` : value.toString()}, Linkage<${keys[0].toString()}>)`)
                        : compatType(specs, value)
                }
            }]);
    }
    function convertStorage$1(specs, registry, { items, prefix }, sectionTypes) {
        return registry.createTypeUnsafe('PalletStorageMetadataV14', [{
                items: items.map(({ docs, fallback, modifier, name, type }) => {
                    let entryType;
                    if (type.isPlain) {
                        const plain = type.asPlain;
                        setTypeOverride(sectionTypes, [plain]);
                        entryType = registry.createTypeUnsafe('StorageEntryTypeV14', [{
                                Plain: compatType(specs, plain)
                            }]);
                    }
                    else if (type.isMap) {
                        const map = type.asMap;
                        entryType = createMapEntry(specs, registry, sectionTypes, {
                            hashers: [map.hasher],
                            isLinked: map.linked.isTrue,
                            isOptional: modifier.isOptional,
                            keys: [map.key],
                            value: map.value
                        });
                    }
                    else if (type.isDoubleMap) {
                        const dm = type.asDoubleMap;
                        entryType = createMapEntry(specs, registry, sectionTypes, {
                            hashers: [dm.hasher, dm.key2Hasher],
                            isLinked: false,
                            isOptional: modifier.isOptional,
                            keys: [dm.key1, dm.key2],
                            value: dm.value
                        });
                    }
                    else {
                        const nm = type.asNMap;
                        entryType = createMapEntry(specs, registry, sectionTypes, {
                            hashers: nm.hashers,
                            isLinked: false,
                            isOptional: modifier.isOptional,
                            keys: nm.keyVec,
                            value: nm.value
                        });
                    }
                    return registry.createTypeUnsafe('StorageEntryMetadataV14', [{
                            docs,
                            fallback,
                            modifier,
                            name,
                            type: entryType
                        }]);
                }),
                prefix
            }]);
    }
    function convertExtrinsic(registry, { signedExtensions, version }) {
        return registry.createTypeUnsafe('ExtrinsicMetadataV14', [{
                signedExtensions: signedExtensions.map((identifier) => ({
                    identifier,
                    type: 0
                })),
                type: 0,
                version
            }]);
    }
    function createPallet(specs, registry, mod, { calls, constants, errors, events, storage }) {
        const sectionTypes = getAliasTypes(registry, util.stringCamelCase(mod.name));
        return registry.createTypeUnsafe('PalletMetadataV14', [{
                calls: calls && convertCalls$1(specs, registry, mod.name, calls, sectionTypes),
                constants: convertConstants$1(specs, registry, constants, sectionTypes),
                errors: errors && convertErrors(specs, registry, mod.name, errors),
                events: events && convertEvents$1(specs, registry, mod.name, events, sectionTypes),
                index: mod.index,
                name: mod.name,
                storage: storage && convertStorage$1(specs, registry, storage, sectionTypes)
            }]);
    }
    function toV14(registry, v13, metaVersion) {
        const specs = [];
        compatTypes(specs, 'Null', 'u8', 'u16', 'u32', 'u64');
        registerOriginCaller(registry, v13.modules, metaVersion);
        const extrinsic = convertExtrinsic(registry, v13.extrinsic);
        const pallets = v13.modules.map((mod) => createPallet(specs, registry, mod, {
            calls: mod.calls.unwrapOr(null),
            constants: mod.constants,
            errors: mod.errors.length ? mod.errors : null,
            events: mod.events.unwrapOr(null),
            storage: mod.storage.unwrapOr(null)
        }));
        return registry.createTypeUnsafe('MetadataV14', [{
                extrinsic,
                lookup: {
                    types: specs.map((type, id) => registry.createTypeUnsafe('PortableType', [{ id, type }]))
                },
                pallets
            }]);
    }

    function toV15(registry, v14, _) {
        const unchecked = v14.lookup.paramTypes.SpRuntimeUncheckedExtrinsic;
        return registry.createTypeUnsafe('MetadataV15', [
            util.objectSpread({}, v14, {
                extrinsic: registry.createTypeUnsafe('ExtrinsicMetadataV15', [
                    util.objectSpread({}, v14.extrinsic, {
                        addressType: unchecked?.[0].type.unwrapOr(0),
                        callType: unchecked?.[1].type.unwrapOr(0),
                        extraType: unchecked?.[3].type.unwrapOr(0),
                        signatureType: unchecked?.[2].type.unwrapOr(0)
                    })
                ]),
                outerEnums: registry.createTypeUnsafe('OuterEnums15', [{
                        callType: unchecked?.[1].type.unwrapOr(0),
                        eventType: v14.lookup.paramTypes.FrameSystemEventRecord?.[0].type.unwrapOr(0)
                    }])
            })
        ]);
    }

    function palletsFromV15(registry, palletV15) {
        const deprecationInfo = registry.createTypeUnsafe('ItemDeprecationInfoV16', ['NotDeprecated']);
        const associatedTypes = registry.createTypeUnsafe('Vec<PalletAssociatedTypeMetadataV16>', [[]]);
        const viewFunctions = registry.createTypeUnsafe('Vec<PalletViewFunctionMetadataV16>', [[]]);
        return registry.createTypeUnsafe('PalletMetadataV16', [
            util.objectSpread({}, palletV15, {
                associatedTypes,
                calls: palletV15.calls.isSome ? convertCalls(registry, palletV15.calls.unwrap()) : null,
                constants: convertConstants(registry, palletV15.constants),
                deprecationInfo,
                errors: palletV15.errors.isSome ? converErrors(registry, palletV15.errors.unwrap()) : null,
                events: palletV15.events.isSome ? convertEvents(registry, palletV15.events.unwrap()) : null,
                storage: palletV15.storage.isSome ? convertStorage(registry, palletV15.storage.unwrap()) : null,
                viewFunctions
            })
        ]);
    }
    function convertStorage(registry, storage) {
        const deprecationInfo = registry.createTypeUnsafe('ItemDeprecationInfoV16', ['NotDeprecated']);
        const items = storage.items.map((item) => registry.createTypeUnsafe('StorageEntryMetadataV16', [
            util.objectSpread({}, item, { deprecationInfo })
        ]));
        return registry.createTypeUnsafe('PalletStorageMetadataV16', [{
                items: registry.createTypeUnsafe('Vec<StorageEntryMetadataV16>', [items]),
                prefix: storage.prefix
            }]);
    }
    function convertCalls(registry, calls) {
        const deprecationInfo = registry.createTypeUnsafe('EnumDeprecationInfoV16', []);
        return registry.createTypeUnsafe('PalletCallMetadataV16', [{
                deprecationInfo,
                type: calls.type
            }]);
    }
    function convertEvents(registry, events) {
        const deprecationInfo = registry.createTypeUnsafe('EnumDeprecationInfoV16', []);
        return registry.createTypeUnsafe('PalletEventMetadataV16', [{
                deprecationInfo,
                type: events.type
            }]);
    }
    function convertConstants(registry, constants) {
        const deprecationInfo = registry.createTypeUnsafe('ItemDeprecationInfoV16', ['NotDeprecated']);
        return registry.createTypeUnsafe('Vec<PalletConstantMetadataV16>', [
            constants.map((constant) => registry.createTypeUnsafe('PalletConstantMetadataV16', [
                util.objectSpread({}, constant, { deprecationInfo })
            ]))
        ]);
    }
    function converErrors(registry, errors) {
        const deprecationInfo = registry.createTypeUnsafe('EnumDeprecationInfoV16', []);
        return registry.createTypeUnsafe('PalletErrorMetadataV16', [{
                deprecationInfo,
                type: errors.type
            }]);
    }
    function extrinsicFromV15(registry, extrinsicV15) {
        const transactionExtensions = extrinsicV15.signedExtensions.map(({ additionalSigned, identifier, type }) => registry.createTypeUnsafe('TransactionExtensionMetadataV16', [{
                identifier,
                implict: additionalSigned,
                type
            }]));
        const indexes = transactionExtensions.map((_, i) => registry.createTypeUnsafe('Compact<u32>', [i]));
        const transactionExtensionsByVersion = registry.createTypeUnsafe('BTreeMap<u8, Vec<Compact<u32>>>', [new Map([[registry.createTypeUnsafe('u8', [extrinsicV15.version]), registry.createTypeUnsafe('Vec<Compact<u32>>', [indexes])]])]);
        return registry.createTypeUnsafe('ExtrinsicMetadataV16', [
            {
                addressType: extrinsicV15.addressType,
                callType: extrinsicV15.callType,
                signatureType: extrinsicV15.signatureType,
                transactionExtensions,
                transactionExtensionsByVersion,
                versions: [extrinsicV15.version]
            }
        ]);
    }
    function apisFromV15(registry, runtimeApiV15) {
        const deprecationInfo = registry.createTypeUnsafe('ItemDeprecationInfoV16', ['NotDeprecated']);
        const methods = runtimeApiV15.methods.map((method) => registry.createTypeUnsafe('RuntimeApiMethodMetadataV16', [util.objectSpread({}, method, { deprecationInfo })]));
        return registry.createTypeUnsafe('RuntimeApiMetadataV16', [
            util.objectSpread({}, runtimeApiV15, {
                deprecationInfo,
                methods,
                version: registry.createTypeUnsafe('Compact<u32>', [0])
            })
        ]);
    }
    function toV16(registry, v15, _) {
        const pallets = v15.pallets.map((pallet) => {
            return palletsFromV15(registry, pallet);
        });
        const extrinsic = extrinsicFromV15(registry, v15.extrinsic);
        const apis = v15.apis.map((runtimeApi) => apisFromV15(registry, runtimeApi));
        return registry.createTypeUnsafe('MetadataV16', [
            util.objectSpread({}, v15, {
                apis,
                extrinsic,
                pallets
            })
        ]);
    }

    function toLatest(_registry, v16, _metaVersion) {
        return v16;
    }

    const MAGIC_NUMBER = 0x6174656d;
    class MagicNumber extends u32 {
        constructor(registry, value) {
            super(registry, value);
            if (!this.isEmpty && !this.eq(MAGIC_NUMBER)) {
                throw new Error(`MagicNumber mismatch: expected ${registry.createTypeUnsafe('u32', [MAGIC_NUMBER]).toHex()}, found ${this.toHex()}`);
            }
        }
    }

    const KNOWN_VERSIONS = [16, 15, 14, 13, 12, 11, 10, 9];
    const LATEST_VERSION = KNOWN_VERSIONS[0];
    const TO_CALLS_VERSION = 14;

    class MetadataVersioned extends Struct {
        #converted = new Map();
        constructor(registry, value) {
            super(registry, {
                magicNumber: MagicNumber,
                metadata: 'MetadataAll'
            }, value);
        }
        #assertVersion = (version) => {
            if (this.version > version) {
                throw new Error(`Cannot convert metadata from version ${this.version} to ${version}`);
            }
            return this.version === version;
        };
        #getVersion = (version, fromPrev) => {
            if (version !== 'latest' && this.#assertVersion(version)) {
                const asCurr = `asV${version}`;
                return this.#metadata()[asCurr];
            }
            if (!this.#converted.has(version)) {
                const asPrev = version === 'latest'
                    ? `asV${LATEST_VERSION}`
                    : `asV${(version - 1)}`;
                this.#converted.set(version, fromPrev(this.registry, this[asPrev], this.version));
            }
            return this.#converted.get(version);
        };
        #metadata = () => {
            return this.getT('metadata');
        };
        get asCallsOnly() {
            return new MetadataVersioned(this.registry, {
                magicNumber: this.magicNumber,
                metadata: this.registry.createTypeUnsafe('MetadataAll', [toCallsOnly(this.registry, this.asLatest), TO_CALLS_VERSION])
            });
        }
        get asV9() {
            this.#assertVersion(9);
            return this.#metadata().asV9;
        }
        get asV10() {
            return this.#getVersion(10, toV10);
        }
        get asV11() {
            return this.#getVersion(11, toV11);
        }
        get asV12() {
            return this.#getVersion(12, toV12);
        }
        get asV13() {
            return this.#getVersion(13, toV13);
        }
        get asV14() {
            return this.#getVersion(14, toV14);
        }
        get asV15() {
            return this.#getVersion(15, toV15);
        }
        get asV16() {
            return this.#getVersion(16, toV16);
        }
        get asLatest() {
            return this.#getVersion('latest', toLatest);
        }
        get magicNumber() {
            return this.getT('magicNumber');
        }
        get version() {
            return this.#metadata().index;
        }
        getUniqTypes(throwError) {
            return getUniqTypes(this.registry, this.asLatest, throwError);
        }
        toJSON() {
            this.asLatest;
            return super.toJSON();
        }
    }

    const EMPTY_METADATA = new Uint8Array([0x6d, 0x65, 0x74, 0x61, 9]);
    const VERSION_IDX = EMPTY_METADATA.length - 1;
    function decodeU8a(registry, u8a) {
        if (u8a.length === 0) {
            return EMPTY_METADATA;
        }
        else if (u8a[VERSION_IDX] === 9) {
            try {
                return new MetadataVersioned(registry, u8a);
            }
            catch {
                u8a[VERSION_IDX] = 10;
            }
        }
        return u8a;
    }
    class Metadata extends MetadataVersioned {
        constructor(registry, value) {
            super(registry, util.isU8a(value) || util.isString(value)
                ? decodeU8a(registry, util.u8aToU8a(value))
                : value);
        }
    }

    function decorateConstants(registry, { pallets }, _version) {
        const result = {};
        for (let i = 0, count = pallets.length; i < count; i++) {
            const { constants, name } = pallets[i];
            if (!constants.isEmpty) {
                util.lazyMethod(result, util.stringCamelCase(name), () => util.lazyMethods({}, constants, (constant) => {
                    const codec = registry.createTypeUnsafe(registry.createLookupType(constant.type), [util.hexToU8a(constant.value.toHex())]);
                    codec.meta = constant;
                    return codec;
                }, objectNameToCamel));
            }
        }
        return result;
    }

    function variantToMeta(lookup, variant) {
        return util.objectSpread({ args: variant.fields.map(({ type }) => lookup.getTypeDef(type).type) }, variant);
    }
    function decorateErrors(registry, { lookup, pallets }, version) {
        const result = {};
        for (let i = 0, count = pallets.length; i < count; i++) {
            const { errors, index, name } = pallets[i];
            if (errors.isSome) {
                const sectionIndex = version >= 12 ? index.toNumber() : i;
                util.lazyMethod(result, util.stringCamelCase(name), () => lazyVariants(lookup, errors.unwrap(), objectNameToString, (variant) => ({
                    is: (errorMod) => util.isCodec(errorMod) &&
                        util.isCodec(errorMod.index) &&
                        errorMod.index.eq(sectionIndex) && (util.isU8a(errorMod.error)
                        ? errorMod.error[0] === variant.index.toNumber()
                        : util.isCodec(errorMod.error) && errorMod.error.eq(variant.index)),
                    meta: registry.createTypeUnsafe('ErrorMetadataLatest', [variantToMeta(lookup, variant)])
                })));
            }
        }
        return result;
    }

    function filterEventsSome({ events }) {
        return events.isSome;
    }
    function decorateEvents(registry, { lookup, pallets }, version) {
        const result = {};
        const filtered = pallets.filter(filterEventsSome);
        for (let i = 0, count = filtered.length; i < count; i++) {
            const { events, index, name } = filtered[i];
            const sectionIndex = version >= 12 ? index.toNumber() : i;
            util.lazyMethod(result, util.stringCamelCase(name), () => lazyVariants(lookup, events.unwrap(), objectNameToString, (variant) => ({
                is: (eventRecord) => util.isCodec(eventRecord) &&
                    util.isU8a(eventRecord.index) &&
                    sectionIndex === eventRecord.index[0] &&
                    variant.index.eq(eventRecord.index[1]),
                meta: registry.createTypeUnsafe('EventMetadataLatest', [variantToMeta(lookup, variant)])
            })));
        }
        return result;
    }

    const DEFAULT_FN = (data) => utilCrypto.xxhashAsU8a(data, 128);
    const HASHERS = {
        Blake2_128: (data) =>
         utilCrypto.blake2AsU8a(data, 128),
        Blake2_128Concat: (data) =>
         util.u8aConcat(utilCrypto.blake2AsU8a(data, 128), util.u8aToU8a(data)),
        Blake2_256: (data) =>
         utilCrypto.blake2AsU8a(data, 256),
        Identity: (data) => util.u8aToU8a(data),
        Twox128: (data) => utilCrypto.xxhashAsU8a(data, 128),
        Twox256: (data) => utilCrypto.xxhashAsU8a(data, 256),
        Twox64Concat: (data) => util.u8aConcat(utilCrypto.xxhashAsU8a(data, 64), util.u8aToU8a(data))
    };
    function getHasher(hasher) {
        return HASHERS[hasher.type] || DEFAULT_FN;
    }

    const NO_RAW_ARGS = {
        args: [],
        hashers: [],
        keys: []
    };
    function filterDefined(a) {
        return !util.isUndefined(a);
    }
    function assertArgs({ method, section }, { args, keys }) {
        if (!Array.isArray(args)) {
            throw new Error(`Call to ${util.stringCamelCase(section || 'unknown')}.${util.stringCamelCase(method || 'unknown')} needs ${keys.length} arguments`);
        }
        else if (args.filter(filterDefined).length !== keys.length) {
            throw new Error(`Call to ${util.stringCamelCase(section || 'unknown')}.${util.stringCamelCase(method || 'unknown')} needs ${keys.length} arguments, found [${args.join(', ')}]`);
        }
    }
    function createKeyRawParts(registry, itemFn, { args, hashers, keys }) {
        const count = keys.length;
        const extra = new Array(count);
        for (let i = 0; i < count; i++) {
            extra[i] = getHasher(hashers[i])(registry.createTypeUnsafe(registry.createLookupType(keys[i]), [args[i]]).toU8a());
        }
        return [
            [
                utilCrypto.xxhashAsU8a(itemFn.prefix, 128),
                utilCrypto.xxhashAsU8a(itemFn.method, 128)
            ],
            extra
        ];
    }
    function createKeyInspect(registry, itemFn, args) {
        assertArgs(itemFn, args);
        const { meta } = itemFn;
        const [prefix, extra] = createKeyRawParts(registry, itemFn, args);
        let types = [];
        if (meta.type.isMap) {
            const { hashers, key } = meta.type.asMap;
            types = hashers.length === 1
                ? [`${hashers[0].type}(${getSiName(registry.lookup, key)})`]
                : registry.lookup.getSiType(key).def.asTuple.map((k, i) => `${hashers[i].type}(${getSiName(registry.lookup, k)})`);
        }
        const names = ['module', 'method'].concat(...args.args.map((_, i) => types[i]));
        return {
            inner: prefix
                .concat(...extra)
                .map((v, i) => ({ name: names[i], outer: [v] }))
        };
    }
    function createKeyRaw(registry, itemFn, args) {
        const [prefix, extra] = createKeyRawParts(registry, itemFn, args);
        return util.u8aConcat(...prefix, ...extra);
    }
    function createKey(registry, itemFn, args) {
        assertArgs(itemFn, args);
        return util.compactAddLength(createKeyRaw(registry, itemFn, args));
    }
    function createStorageInspect(registry, itemFn, options) {
        const { meta: { type } } = itemFn;
        return (...args) => {
            if (type.isPlain) {
                return options.skipHashing
                    ? { inner: [], name: 'wellKnown', outer: [util.u8aToU8a(options.key)] }
                    : createKeyInspect(registry, itemFn, NO_RAW_ARGS);
            }
            const { hashers, key } = type.asMap;
            return hashers.length === 1
                ? createKeyInspect(registry, itemFn, { args, hashers, keys: [key] })
                : createKeyInspect(registry, itemFn, { args, hashers, keys: registry.lookup.getSiType(key).def.asTuple });
        };
    }
    function createStorageFn(registry, itemFn, options) {
        const { meta: { type } } = itemFn;
        let cacheKey = null;
        return (...args) => {
            if (type.isPlain) {
                if (!cacheKey) {
                    cacheKey = options.skipHashing
                        ? util.compactAddLength(util.u8aToU8a(options.key))
                        : createKey(registry, itemFn, NO_RAW_ARGS);
                }
                return cacheKey;
            }
            const { hashers, key } = type.asMap;
            return hashers.length === 1
                ? createKey(registry, itemFn, { args, hashers, keys: [key] })
                : createKey(registry, itemFn, { args, hashers, keys: registry.lookup.getSiType(key).def.asTuple });
        };
    }
    function createWithMeta(registry, itemFn, options) {
        const { meta, method, prefix, section } = itemFn;
        const storageFn = createStorageFn(registry, itemFn, options);
        storageFn.inspect = createStorageInspect(registry, itemFn, options);
        storageFn.meta = meta;
        storageFn.method = util.stringCamelCase(method);
        storageFn.prefix = prefix;
        storageFn.section = section;
        storageFn.toJSON = () => util.objectSpread({ storage: { method, prefix, section } }, meta.toJSON());
        return storageFn;
    }
    function extendHeadMeta(registry, { meta: { docs, name, type }, section }, { method }, iterFn) {
        const meta = registry.createTypeUnsafe('StorageEntryMetadataLatest', [{
                docs,
                fallback: registry.createTypeUnsafe('Bytes', []),
                modifier: registry.createTypeUnsafe('StorageEntryModifierLatest', [1]),
                name,
                type: registry.createTypeUnsafe('StorageEntryTypeLatest', [type.asMap.key, 0])
            }]);
        iterFn.meta = meta;
        const fn = (...args) => registry.createTypeUnsafe('StorageKey', [iterFn(...args), { method, section }]);
        fn.meta = meta;
        return fn;
    }
    function extendPrefixedMap(registry, itemFn, storageFn) {
        const { meta: { type }, method, section } = itemFn;
        storageFn.iterKey = extendHeadMeta(registry, itemFn, storageFn, (...args) => {
            if (args.length && (type.isPlain || (args.length >= type.asMap.hashers.length))) {
                throw new Error(`Iteration of ${util.stringCamelCase(section || 'unknown')}.${util.stringCamelCase(method || 'unknown')} needs arguments to be at least one less than the full arguments, found [${args.join(', ')}]`);
            }
            if (args.length) {
                if (type.isMap) {
                    const { hashers, key } = type.asMap;
                    const keysVec = hashers.length === 1
                        ? [key]
                        : registry.lookup.getSiType(key).def.asTuple;
                    return new Raw(registry, createKeyRaw(registry, itemFn, { args, hashers: hashers.slice(0, args.length), keys: keysVec.slice(0, args.length) }));
                }
            }
            return new Raw(registry, createKeyRaw(registry, itemFn, NO_RAW_ARGS));
        });
        return storageFn;
    }
    function createFunction(registry, itemFn, options) {
        const { meta: { type } } = itemFn;
        const storageFn = createWithMeta(registry, itemFn, options);
        if (type.isMap) {
            extendPrefixedMap(registry, itemFn, storageFn);
        }
        storageFn.keyPrefix = (...args) => (storageFn.iterKey && storageFn.iterKey(...args)) ||
            util.compactStripLength(storageFn())[1];
        return storageFn;
    }

    function findSiPrimitive(registry, type) {
        const prim = type.toLowerCase();
        return registry.lookup.types.find((t) => (t.type.def.isPrimitive &&
            t.type.def.asPrimitive.toString().toLowerCase() === prim) || (t.type.def.isHistoricMetaCompat &&
            t.type.def.asHistoricMetaCompat.toString().toLowerCase() === prim));
    }
    function findSiType(registry, type) {
        let portable = findSiPrimitive(registry, type);
        if (!portable && (type === 'Bytes' || type.startsWith('[u8;'))) {
            const u8 = findSiPrimitive(registry, 'u8');
            if (u8) {
                if (type === 'Bytes') {
                    portable = registry.lookup.types.find((t) => (t.type.def.isSequence &&
                        t.type.def.asSequence.type.eq(u8.id)) || (t.type.def.isHistoricMetaCompat &&
                        t.type.def.asHistoricMetaCompat.eq(type)));
                }
                else {
                    const td = getTypeDef(type);
                    portable = registry.lookup.types.find((t) => (t.type.def.isArray &&
                        t.type.def.asArray.eq({
                            len: td.length,
                            type: u8.id
                        })) || (t.type.def.isHistoricMetaCompat &&
                        t.type.def.asHistoricMetaCompat.eq(type)));
                }
            }
        }
        if (!portable) {
            console.warn(`Unable to map ${type} to a lookup index`);
        }
        return portable;
    }
    function createRuntimeFunction({ method, prefix, section }, key, { docs, type }) {
        return (registry) => createFunction(registry, {
            meta: registry.createTypeUnsafe('StorageEntryMetadataLatest', [{
                    docs: registry.createTypeUnsafe('Vec<Text>', [[docs]]),
                    modifier: registry.createTypeUnsafe('StorageEntryModifierLatest', ['Required']),
                    name: registry.createTypeUnsafe('Text', [method]),
                    toJSON: () => key,
                    type: registry.createTypeUnsafe('StorageEntryTypeLatest', [{ Plain: findSiType(registry, type)?.id || 0 }])
                }]),
            method,
            prefix,
            section
        }, { key, skipHashing: true });
    }

    const prefix = 'Substrate';
    const section = 'substrate';
    function createSubstrateFn(method, key, meta) {
        return createRuntimeFunction({ method, prefix, section }, key, meta);
    }
    const substrate = {
        changesTrieConfig: createSubstrateFn('changesTrieConfig', ':changes_trie', {
            docs: 'Changes trie configuration is stored under this key.',
            type: 'u32'
        }),
        childStorageKeyPrefix: createSubstrateFn('childStorageKeyPrefix', ':child_storage:', {
            docs: 'Prefix of child storage keys.',
            type: 'u32'
        }),
        code: createSubstrateFn('code', ':code', {
            docs: 'Wasm code of the runtime.',
            type: 'Bytes'
        }),
        defaultChildStorageKeyPrefix: createSubstrateFn('defaultChildStorageKeyPrefix', ':child_storage:default:', {
            docs: 'Prefix of the default child storage keys in the top trie.',
            type: 'u32'
        }),
        extrinsicIndex: createSubstrateFn('extrinsicIndex', ':extrinsic_index', {
            docs: 'Current extrinsic index (u32) is stored under this key.',
            type: 'u32'
        }),
        heapPages: createSubstrateFn('heapPages', ':heappages', {
            docs: 'Number of wasm linear memory pages required for execution of the runtime.',
            type: 'u64'
        }),
        intrablockEntropy: createSubstrateFn('intrablockEntropy', ':intrablock_entropy', {
            docs: 'Current intra-block entropy (a universally unique `[u8; 32]` value) is stored here.',
            type: '[u8; 32]'
        }),
        storageVersionStorageKeyPostfix: createSubstrateFn('storageVersionStorageKeyPostfix', ':__STORAGE_VERSION__:', {
            docs: 'The storage key postfix that is used to store the [`StorageVersion`] per pallet.',
            type: 'u16'
        }),
        transactionLevelKey: createSubstrateFn('transactionLevelKey', ':transaction_level:', {
            docs: 'The key that holds the current number of active layers.',
            type: 'u32'
        })
    };

    function getStorage(registry) {
        const storage = {};
        const entries = Object.entries(substrate);
        for (let e = 0, count = entries.length; e < count; e++) {
            storage[entries[e][0]] = entries[e][1](registry);
        }
        return { substrate: storage };
    }

    const VERSION_NAME = 'palletVersion';
    const VERSION_KEY = ':__STORAGE_VERSION__:';
    const VERSION_DOCS = { docs: 'Returns the current pallet version from storage', type: 'u16' };
    function decorateStorage(registry, { pallets }, _metaVersion) {
        const result = getStorage(registry);
        for (let i = 0, count = pallets.length; i < count; i++) {
            const { name, storage } = pallets[i];
            if (storage.isSome) {
                const section = util.stringCamelCase(name);
                const { items, prefix: _prefix } = storage.unwrap();
                const prefix = _prefix.toString();
                util.lazyMethod(result, section, () => util.lazyMethods({
                    palletVersion: createRuntimeFunction({ method: VERSION_NAME, prefix, section }, createKeyRaw(registry, { method: VERSION_KEY, prefix: name.toString() }, NO_RAW_ARGS), VERSION_DOCS)(registry)
                }, items, (meta) => createFunction(registry, { meta, method: meta.name.toString(), prefix, section }, {}), objectNameToCamel));
            }
        }
        return result;
    }

    function expandMetadata(registry, metadata) {
        if (!(metadata instanceof Metadata)) {
            throw new Error('You need to pass a valid Metadata instance to Decorated');
        }
        const latest = metadata.asLatest;
        const version = metadata.version;
        return {
            consts: decorateConstants(registry, latest),
            errors: decorateErrors(registry, latest, version),
            events: decorateEvents(registry, latest, version),
            query: decorateStorage(registry, latest),
            registry,
            tx: decorateExtrinsics(registry, latest, version)
        };
    }

    const DEFAULT_FIRST_CALL_IDX = new Uint8Array(2);
    const l = util.logger('registry');
    function sortDecimalStrings(a, b) {
        return parseInt(a, 10) - parseInt(b, 10);
    }
    function valueToString(v) {
        return v.toString();
    }
    function getFieldArgs(lookup, fields) {
        const count = fields.length;
        const args = new Array(count);
        for (let i = 0; i < count; i++) {
            args[i] = lookup.getTypeDef(fields[i].type).type;
        }
        return args;
    }
    function clearRecord(record) {
        const keys = Object.keys(record);
        for (let i = 0, count = keys.length; i < count; i++) {
            delete record[keys[i]];
        }
    }
    function getVariantStringIdx({ index }) {
        return index.toString();
    }
    function injectErrors(_, { lookup, pallets }, version, result) {
        clearRecord(result);
        for (let i = 0, count = pallets.length; i < count; i++) {
            const { errors, index, name } = pallets[i];
            if (errors.isSome) {
                const sectionName = util.stringCamelCase(name);
                util.lazyMethod(result, version >= 12 ? index.toNumber() : i, () => lazyVariants(lookup, errors.unwrap(), getVariantStringIdx, ({ docs, fields, index, name }) => ({
                    args: getFieldArgs(lookup, fields),
                    docs: docs.map(valueToString),
                    fields,
                    index: index.toNumber(),
                    method: name.toString(),
                    name: name.toString(),
                    section: sectionName
                })));
            }
        }
    }
    function injectEvents(registry, { lookup, pallets }, version, result) {
        const filtered = pallets.filter(filterEventsSome);
        clearRecord(result);
        for (let i = 0, count = filtered.length; i < count; i++) {
            const { events, index, name } = filtered[i];
            util.lazyMethod(result, version >= 12 ? index.toNumber() : i, () => lazyVariants(lookup, events.unwrap(), getVariantStringIdx, (variant) => {
                const meta = registry.createType('EventMetadataLatest', util.objectSpread({}, variant, { args: getFieldArgs(lookup, variant.fields) }));
                return class extends GenericEventData {
                    constructor(registry, value) {
                        super(registry, value, meta, util.stringCamelCase(name), variant.name.toString());
                    }
                };
            }));
        }
    }
    function injectExtrinsics(registry, { lookup, pallets }, version, result, mapping) {
        const filtered = pallets.filter(filterCallsSome);
        clearRecord(result);
        clearRecord(mapping);
        for (let i = 0, count = filtered.length; i < count; i++) {
            const { calls, index, name } = filtered[i];
            const sectionIndex = version >= 12 ? index.toNumber() : i;
            const sectionName = util.stringCamelCase(name);
            const allCalls = calls.unwrap();
            util.lazyMethod(result, sectionIndex, () => lazyVariants(lookup, allCalls, getVariantStringIdx, (variant) => createCallFunction(registry, lookup, variant, sectionName, sectionIndex)));
            const { path } = registry.lookup.getSiType(allCalls.type);
            const palletIdx = path.findIndex((v) => v.eq('pallet'));
            if (palletIdx !== -1) {
                const name = util.stringCamelCase(path
                    .slice(0, palletIdx)
                    .map((p, i) => i === 0
                    ? p.replace(/^(frame|pallet)_/, '')
                    : p)
                    .join(' '));
                if (!mapping[name]) {
                    mapping[name] = [sectionName];
                }
                else {
                    mapping[name].push(sectionName);
                }
            }
        }
    }
    function extractProperties(registry, metadata) {
        const original = registry.getChainProperties();
        const constants = decorateConstants(registry, metadata.asLatest, metadata.version);
        const ss58Format = constants['system'] && (constants['system']['sS58Prefix'] || constants['system']['ss58Prefix']);
        if (!ss58Format) {
            return original;
        }
        const { isEthereum, tokenDecimals, tokenSymbol } = original || {};
        return registry.createTypeUnsafe('ChainProperties', [{ isEthereum, ss58Format, tokenDecimals, tokenSymbol }]);
    }
    class TypeRegistry {
        #chainProperties;
        #classes = new Map();
        #definitions = new Map();
        #firstCallIndex = null;
        #hasher = utilCrypto.blake2AsU8a;
        #knownTypes = {};
        #lookup;
        #metadata;
        #metadataVersion = 0;
        #signedExtensions = fallbackExtensions;
        #unknownTypes = new Map();
        #userExtensions;
        #knownDefaults;
        #knownDefaultsEntries;
        #knownDefinitions;
        #metadataCalls = {};
        #metadataErrors = {};
        #metadataEvents = {};
        #moduleMap = {};
        createdAtHash;
        constructor(createdAtHash) {
            this.#knownDefaults = new Map(Object.entries({ Json, Metadata, PortableRegistry, Raw, ...baseTypes }));
            this.#knownDefaultsEntries = Array.from(this.#knownDefaults.entries());
            this.#knownDefinitions = definitions;
            const allKnown = Object.values(this.#knownDefinitions);
            for (let i = 0, count = allKnown.length; i < count; i++) {
                this.register(allKnown[i].types);
            }
            if (createdAtHash) {
                this.createdAtHash = this.createType('BlockHash', createdAtHash);
            }
        }
        get chainDecimals() {
            if (this.#chainProperties?.tokenDecimals.isSome) {
                const allDecimals = this.#chainProperties.tokenDecimals.unwrap();
                if (allDecimals.length) {
                    return allDecimals.map((b) => b.toNumber());
                }
            }
            return [12];
        }
        get chainIsEthereum() {
            return this.#chainProperties?.isEthereum.isTrue || false;
        }
        get chainSS58() {
            return this.#chainProperties?.ss58Format.isSome
                ? this.#chainProperties.ss58Format.unwrap().toNumber()
                : undefined;
        }
        get chainTokens() {
            if (this.#chainProperties?.tokenSymbol.isSome) {
                const allTokens = this.#chainProperties.tokenSymbol.unwrap();
                if (allTokens.length) {
                    return allTokens.map(valueToString);
                }
            }
            return [util.formatBalance.getDefaults().unit];
        }
        get firstCallIndex() {
            return this.#firstCallIndex || DEFAULT_FIRST_CALL_IDX;
        }
        isLookupType(value) {
            return /Lookup\d+$/.test(value);
        }
        createLookupType(lookupId) {
            return `Lookup${typeof lookupId === 'number' ? lookupId : lookupId.toNumber()}`;
        }
        get knownTypes() {
            return this.#knownTypes;
        }
        get lookup() {
            return util.assertReturn(this.#lookup, 'PortableRegistry has not been set on this registry');
        }
        get metadata() {
            return util.assertReturn(this.#metadata, 'Metadata has not been set on this registry');
        }
        get unknownTypes() {
            return [...this.#unknownTypes.keys()];
        }
        get signedExtensions() {
            return this.#signedExtensions;
        }
        clearCache() {
            this.#classes = new Map();
        }
        createClass(type) {
            return createClassUnsafe(this, type);
        }
        createClassUnsafe(type) {
            return createClassUnsafe(this, type);
        }
        createType(type, ...params) {
            return createTypeUnsafe(this, type, params);
        }
        createTypeUnsafe(type, params, options) {
            return createTypeUnsafe(this, type, params, options);
        }
        findMetaCall(callIndex) {
            const [section, method] = [callIndex[0], callIndex[1]];
            return util.assertReturn(this.#metadataCalls[`${section}`] && this.#metadataCalls[`${section}`][`${method}`], () => `findMetaCall: Unable to find Call with index [${section}, ${method}]/[${callIndex.toString()}]`);
        }
        findMetaError(errorIndex) {
            const [section, method] = util.isU8a(errorIndex)
                ? [errorIndex[0], errorIndex[1]]
                : [
                    errorIndex.index.toNumber(),
                    util.isU8a(errorIndex.error)
                        ? errorIndex.error[0]
                        : errorIndex.error.toNumber()
                ];
            return util.assertReturn(this.#metadataErrors[`${section}`] && this.#metadataErrors[`${section}`][`${method}`], () => `findMetaError: Unable to find Error with index [${section}, ${method}]/[${errorIndex.toString()}]`);
        }
        findMetaEvent(eventIndex) {
            const [section, method] = [eventIndex[0], eventIndex[1]];
            return util.assertReturn(this.#metadataEvents[`${section}`] && this.#metadataEvents[`${section}`][`${method}`], () => `findMetaEvent: Unable to find Event with index [${section}, ${method}]/[${eventIndex.toString()}]`);
        }
        get(name, withUnknown, knownTypeDef) {
            return this.getUnsafe(name, withUnknown, knownTypeDef);
        }
        getUnsafe(name, withUnknown, knownTypeDef) {
            let Type = this.#classes.get(name) || this.#knownDefaults.get(name);
            if (!Type) {
                const definition = this.#definitions.get(name);
                let BaseType;
                if (definition) {
                    BaseType = createClassUnsafe(this, definition);
                }
                else if (knownTypeDef) {
                    BaseType = constructTypeClass(this, knownTypeDef);
                }
                else if (withUnknown) {
                    l.warn(`Unable to resolve type ${name}, it will fail on construction`);
                    this.#unknownTypes.set(name, true);
                    BaseType = DoNotConstruct.with(name);
                }
                if (BaseType) {
                    Type = class extends BaseType {
                    };
                    this.#classes.set(name, Type);
                    if (knownTypeDef && util.isNumber(knownTypeDef.lookupIndex)) {
                        this.#classes.set(this.createLookupType(knownTypeDef.lookupIndex), Type);
                    }
                }
            }
            return Type;
        }
        getChainProperties() {
            return this.#chainProperties;
        }
        getClassName(Type) {
            const names = [];
            for (const [name, Clazz] of this.#knownDefaultsEntries) {
                if (Type === Clazz) {
                    names.push(name);
                }
            }
            for (const [name, Clazz] of this.#classes.entries()) {
                if (Type === Clazz) {
                    names.push(name);
                }
            }
            return names.length
                ? names.sort().reverse()[0]
                : undefined;
        }
        getDefinition(typeName) {
            return this.#definitions.get(typeName);
        }
        getModuleInstances(specName, moduleName) {
            return this.#knownTypes?.typesBundle?.spec?.[specName.toString()]?.instances?.[moduleName] || this.#moduleMap[moduleName];
        }
        getOrThrow(name) {
            const Clazz = this.get(name);
            if (!Clazz) {
                throw new Error(`type ${name} not found`);
            }
            return Clazz;
        }
        getOrUnknown(name) {
            return this.get(name, true);
        }
        getTransactionExtensionVersion() {
            return 0;
        }
        getSignedExtensionExtra() {
            return expandExtensionTypes(this.#signedExtensions, 'payload', this.#userExtensions);
        }
        getSignedExtensionTypes() {
            return expandExtensionTypes(this.#signedExtensions, 'extrinsic', this.#userExtensions);
        }
        hasClass(name) {
            return this.#classes.has(name) || !!this.#knownDefaults.has(name);
        }
        hasDef(name) {
            return this.#definitions.has(name);
        }
        hasType(name) {
            return !this.#unknownTypes.get(name) && (this.hasClass(name) || this.hasDef(name));
        }
        hash(data) {
            return this.createType('CodecHash', this.#hasher(data));
        }
        register(arg1, arg2) {
            if (util.isFunction(arg1)) {
                this.#classes.set(arg1.name, arg1);
            }
            else if (util.isString(arg1)) {
                if (!util.isFunction(arg2)) {
                    throw new Error(`Expected class definition passed to '${arg1}' registration`);
                }
                else if (arg1 === arg2.toString()) {
                    throw new Error(`Unable to register circular ${arg1} === ${arg1}`);
                }
                this.#classes.set(arg1, arg2);
            }
            else {
                this.#registerObject(arg1);
            }
        }
        #registerObject = (obj) => {
            const entries = Object.entries(obj);
            for (let e = 0, count = entries.length; e < count; e++) {
                const [name, type] = entries[e];
                if (util.isFunction(type)) {
                    this.#classes.set(name, type);
                }
                else {
                    const def = util.isString(type)
                        ? type
                        : util.stringify(type);
                    if (name === def) {
                        throw new Error(`Unable to register circular ${name} === ${def}`);
                    }
                    if (this.#classes.has(name)) {
                        this.#classes.delete(name);
                    }
                    this.#definitions.set(name, def);
                }
            }
        };
        setChainProperties(properties) {
            if (properties) {
                this.#chainProperties = properties;
            }
        }
        setHasher(hasher) {
            this.#hasher = hasher || utilCrypto.blake2AsU8a;
        }
        setKnownTypes(knownTypes) {
            this.#knownTypes = knownTypes;
        }
        setLookup(lookup) {
            this.#lookup = lookup;
            lookup.register();
        }
        #registerLookup = (lookup) => {
            this.setLookup(lookup);
            let Weight = null;
            if (this.hasType('SpWeightsWeightV2Weight')) {
                const weightv2 = this.createType('SpWeightsWeightV2Weight');
                Weight = weightv2.refTime && weightv2.proofSize
                    ? 'SpWeightsWeightV2Weight'
                    : 'WeightV1';
            }
            else if (!util.isBn(this.createType('Weight'))) {
                Weight = 'WeightV1';
            }
            if (Weight) {
                this.register({ Weight });
            }
        };
        setMetadata(metadata, signedExtensions, userExtensions, noInitWarn) {
            this.#metadata = metadata.asLatest;
            this.#metadataVersion = metadata.version;
            this.#firstCallIndex = null;
            this.#registerLookup(this.#metadata.lookup);
            injectExtrinsics(this, this.#metadata, this.#metadataVersion, this.#metadataCalls, this.#moduleMap);
            injectErrors(this, this.#metadata, this.#metadataVersion, this.#metadataErrors);
            injectEvents(this, this.#metadata, this.#metadataVersion, this.#metadataEvents);
            const [defSection] = Object
                .keys(this.#metadataCalls)
                .sort(sortDecimalStrings);
            if (defSection) {
                const [defMethod] = Object
                    .keys(this.#metadataCalls[defSection])
                    .sort(sortDecimalStrings);
                if (defMethod) {
                    this.#firstCallIndex = new Uint8Array([parseInt(defSection, 10), parseInt(defMethod, 10)]);
                }
            }
            this.setSignedExtensions(signedExtensions || (this.#metadata.extrinsic.versions.length > 0 && this.#metadata.extrinsic.versions.every((value) => value > 0)
                ? this.#metadata.extrinsic.transactionExtensions.map(({ identifier }) => identifier.toString())
                : fallbackExtensions), userExtensions, noInitWarn);
            this.setChainProperties(extractProperties(this, metadata));
        }
        setSignedExtensions(signedExtensions = fallbackExtensions, userExtensions, noInitWarn) {
            this.#signedExtensions = signedExtensions;
            this.#userExtensions = userExtensions;
            if (!noInitWarn) {
                const unknown = findUnknownExtensions(this.#signedExtensions, this.#userExtensions);
                if (unknown.length) {
                    l.warn(`Unknown signed extensions ${unknown.join(', ')} found, treating them as no-effect`);
                }
            }
        }
    }

    exports.BTreeMap = BTreeMap;
    exports.BTreeSet = BTreeSet;
    exports.BitVec = BitVec;
    exports.Bool = bool;
    exports.Bytes = Bytes;
    exports.CodecMap = CodecMap;
    exports.CodecSet = CodecSet;
    exports.Compact = Compact;
    exports.Data = Data;
    exports.DoNotConstruct = DoNotConstruct;
    exports.Enum = Enum;
    exports.F32 = f32;
    exports.F64 = f64;
    exports.GeneralExtrinsic = GeneralExtrinsic;
    exports.GenericAccountId = GenericAccountId;
    exports.GenericAccountId32 = GenericAccountId;
    exports.GenericAccountId33 = GenericAccountId33;
    exports.GenericAccountIndex = GenericAccountIndex;
    exports.GenericAddress = GenericMultiAddress;
    exports.GenericBlock = GenericBlock;
    exports.GenericCall = GenericCall;
    exports.GenericChainProperties = GenericChainProperties;
    exports.GenericConsensusEngineId = GenericConsensusEngineId;
    exports.GenericEthereumAccountId = GenericEthereumAccountId;
    exports.GenericEthereumLookupSource = GenericEthereumLookupSource;
    exports.GenericEvent = GenericEvent;
    exports.GenericEventData = GenericEventData;
    exports.GenericExtrinsic = GenericExtrinsic;
    exports.GenericExtrinsicEra = GenericExtrinsicEra;
    exports.GenericExtrinsicPayload = GenericExtrinsicPayload;
    exports.GenericExtrinsicPayloadUnknown = GenericExtrinsicPayloadUnknown;
    exports.GenericExtrinsicPayloadV4 = GenericExtrinsicPayloadV4;
    exports.GenericExtrinsicPayloadV5 = GenericExtrinsicPayloadV5;
    exports.GenericExtrinsicSignatureV4 = GenericExtrinsicSignatureV4;
    exports.GenericExtrinsicSignatureV5 = GenericExtrinsicSignatureV5;
    exports.GenericExtrinsicUnknown = GenericExtrinsicUnknown;
    exports.GenericExtrinsicV4 = GenericExtrinsicV4;
    exports.GenericExtrinsicV5 = GenericExtrinsicV5;
    exports.GenericImmortalEra = ImmortalEra;
    exports.GenericLookupSource = GenericLookupSource;
    exports.GenericMortalEra = MortalEra;
    exports.GenericMultiAddress = GenericMultiAddress;
    exports.GenericSignerPayload = GenericSignerPayload;
    exports.GenericVote = GenericVote;
    exports.HashMap = HashMap;
    exports.I128 = i128;
    exports.I16 = i16;
    exports.I256 = i256;
    exports.I32 = i32;
    exports.I64 = i64;
    exports.I8 = i8;
    exports.ISize = isize;
    exports.Int = Int;
    exports.Json = Json;
    exports.Linkage = Linkage;
    exports.Map = CodecMap;
    exports.Metadata = Metadata;
    exports.Null = Null;
    exports.Option = Option;
    exports.OptionBool = OptionBool;
    exports.PortableRegistry = PortableRegistry;
    exports.Range = Range;
    exports.RangeInclusive = RangeInclusive;
    exports.Raw = Raw;
    exports.Result = Result;
    exports.Set = CodecSet;
    exports.StorageKey = StorageKey;
    exports.Struct = Struct;
    exports.Text = Text;
    exports.Tuple = Tuple;
    exports.Type = Type;
    exports.TypeRegistry = TypeRegistry;
    exports.U128 = u128;
    exports.U16 = u16;
    exports.U256 = u256;
    exports.U32 = u32;
    exports.U64 = u64;
    exports.U8 = u8;
    exports.U8aFixed = U8aFixed;
    exports.UInt = UInt;
    exports.USize = usize;
    exports.Vec = Vec;
    exports.VecFixed = VecFixed;
    exports.WrapperKeepOpaque = WrapperKeepOpaque;
    exports.WrapperOpaque = WrapperOpaque;
    exports.XCM_MAPPINGS = XCM_MAPPINGS;
    exports.bool = bool;
    exports.constructTypeClass = constructTypeClass;
    exports.convertSiV0toV1 = toV1;
    exports.createClass = createClass;
    exports.createClassUnsafe = createClassUnsafe;
    exports.createType = createType;
    exports.createTypeUnsafe = createTypeUnsafe;
    exports.decorateConstants = decorateConstants;
    exports.decorateExtrinsics = decorateExtrinsics;
    exports.decorateStorage = decorateStorage;
    exports.encodeTypeDef = encodeTypeDef;
    exports.expandMetadata = expandMetadata;
    exports.f32 = f32;
    exports.f64 = f64;
    exports.getTypeClass = getTypeClass;
    exports.getTypeDef = getTypeDef;
    exports.i128 = i128;
    exports.i16 = i16;
    exports.i256 = i256;
    exports.i32 = i32;
    exports.i64 = i64;
    exports.i8 = i8;
    exports.isize = isize;
    exports.lazyVariants = lazyVariants;
    exports.mapXcmTypes = mapXcmTypes;
    exports.packageInfo = packageInfo;
    exports.paramsNotation = paramsNotation;
    exports.rpcDefinitions = jsonrpc$1;
    exports.typeDefinitions = definitions;
    exports.typeSplit = typeSplit;
    exports.u128 = u128;
    exports.u16 = u16;
    exports.u256 = u256;
    exports.u32 = u32;
    exports.u64 = u64;
    exports.u8 = u8;
    exports.unwrapStorageType = unwrapStorageType;
    exports.usize = usize;
    exports.withTypeString = withTypeString;

}));
