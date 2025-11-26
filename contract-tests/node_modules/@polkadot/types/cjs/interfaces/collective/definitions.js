"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = {
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
