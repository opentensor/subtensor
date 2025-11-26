"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.runtime = void 0;
exports.runtime = {
    StakingApi: [
        {
            methods: {
                nominations_quota: {
                    description: 'Returns the nominations quota for a nominator with a given balance.',
                    params: [
                        {
                            name: 'balance',
                            type: 'Balance'
                        }
                    ],
                    type: 'u32'
                }
            },
            version: 1
        }
    ]
};
