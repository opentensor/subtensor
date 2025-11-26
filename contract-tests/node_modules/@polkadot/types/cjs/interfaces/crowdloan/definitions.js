"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = {
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
