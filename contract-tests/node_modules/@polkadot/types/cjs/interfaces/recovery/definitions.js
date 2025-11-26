"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = {
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
