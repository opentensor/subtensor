"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.substrate = void 0;
const emptyCheck_js_1 = require("./emptyCheck.js");
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
exports.substrate = {
    ChargeTransactionPayment,
    CheckBlockGasLimit: emptyCheck_js_1.emptyCheck,
    CheckEra: CheckMortality,
    CheckGenesis: {
        extrinsic: {},
        payload: {
            genesisHash: 'Hash'
        }
    },
    CheckMetadataHash,
    CheckMortality,
    CheckNonZeroSender: emptyCheck_js_1.emptyCheck,
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
    CheckWeight: emptyCheck_js_1.emptyCheck,
    LockStakingStatus: emptyCheck_js_1.emptyCheck,
    SkipCheckIfFeeless: ChargeTransactionPayment,
    ValidateEquivocationReport: emptyCheck_js_1.emptyCheck,
    WeightReclaim: emptyCheck_js_1.emptyCheck
};
