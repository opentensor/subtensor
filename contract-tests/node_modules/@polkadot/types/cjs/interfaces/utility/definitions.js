"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = {
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
