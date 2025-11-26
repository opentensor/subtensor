"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.runtime = void 0;
exports.runtime = {
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
