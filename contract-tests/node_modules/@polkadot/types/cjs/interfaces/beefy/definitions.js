"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const rpc_js_1 = require("./rpc.js");
const runtime_js_1 = require("./runtime.js");
exports.default = {
    rpc: rpc_js_1.rpc,
    runtime: runtime_js_1.runtime,
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
