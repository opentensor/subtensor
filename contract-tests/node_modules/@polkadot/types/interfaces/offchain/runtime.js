export const runtime = {
    OffchainWorkerApi: [
        {
            methods: {
                offchain_worker: {
                    description: 'Starts the off-chain task for given block header.',
                    params: [
                        {
                            name: 'header',
                            type: 'Header'
                        }
                    ],
                    type: 'Null'
                }
            },
            version: 2
        },
        {
            methods: {
                offchain_worker: {
                    description: 'Starts the off-chain task for given block header.',
                    params: [
                        {
                            name: 'number',
                            type: 'BlockNumber'
                        }
                    ],
                    type: 'Null'
                }
            },
            version: 1
        }
    ]
};
