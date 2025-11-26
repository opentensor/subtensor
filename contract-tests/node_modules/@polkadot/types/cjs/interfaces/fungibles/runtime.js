"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.runtime = void 0;
exports.runtime = {
    FungiblesApi: [
        {
            methods: {
                query_account_balances: {
                    description: 'Returns the list of all `MultiAsset` that an `AccountId` has',
                    params: [
                        {
                            name: 'account',
                            type: 'AccountId'
                        }
                    ],
                    type: 'Result<Vec<XcmV3MultiAsset>, FungiblesAccessError>'
                }
            },
            version: 1
        },
        {
            methods: {
                query_account_balances: {
                    description: 'Returns the list of all `MultiAsset` that an `AccountId` has',
                    params: [
                        {
                            name: 'account',
                            type: 'AccountId'
                        }
                    ],
                    type: 'Result<XcmVersionedAssets, FungiblesAccessError>'
                }
            },
            version: 2
        }
    ]
};
