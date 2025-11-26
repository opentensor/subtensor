import { emptyCheck } from './emptyCheck.js';
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
export const substrate = {
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
