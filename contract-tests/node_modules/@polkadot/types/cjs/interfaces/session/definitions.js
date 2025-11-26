"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const runtime_js_1 = require("./runtime.js");
const keyTypes = {
    // key for beefy
    BeefyKey: '[u8; 33]',
    // default to Substrate master defaults, 4 keys (polkadot master, 5 keys)
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
exports.default = {
    rpc: {},
    runtime: runtime_js_1.runtime,
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
