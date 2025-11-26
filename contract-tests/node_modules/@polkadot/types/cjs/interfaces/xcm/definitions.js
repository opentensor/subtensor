"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const types_create_1 = require("@polkadot/types-create");
const v0_js_1 = require("./v0.js");
const v1_js_1 = require("./v1.js");
const v2_js_1 = require("./v2.js");
const v3_js_1 = require("./v3.js");
const v4_js_1 = require("./v4.js");
const v5_js_1 = require("./v5.js");
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
exports.default = {
    rpc: {},
    types: {
        ...location,
        ...xcm,
        ...v0_js_1.v0,
        ...v1_js_1.v1,
        ...v2_js_1.v2,
        ...v3_js_1.v3,
        ...v4_js_1.v4,
        ...v5_js_1.v5,
        ...(0, types_create_1.mapXcmTypes)(XCM_LATEST),
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
