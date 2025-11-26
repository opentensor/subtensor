export const runtime = {
    TaggedTransactionQueue: [
        {
            methods: {
                validate_transaction: {
                    description: 'Validate the transaction.',
                    params: [
                        {
                            name: 'source',
                            type: 'TransactionSource'
                        },
                        {
                            name: 'tx',
                            type: 'Extrinsic'
                        },
                        {
                            name: 'blockHash',
                            type: 'BlockHash'
                        }
                    ],
                    type: 'TransactionValidity'
                }
            },
            version: 3
        },
        {
            methods: {
                validate_transaction: {
                    description: 'Validate the transaction.',
                    params: [
                        {
                            name: 'source',
                            type: 'TransactionSource'
                        },
                        {
                            name: 'tx',
                            type: 'Extrinsic'
                        }
                    ],
                    type: 'TransactionValidity'
                }
            },
            version: 2
        },
        {
            methods: {
                validate_transaction: {
                    description: 'Validate the transaction.',
                    params: [
                        {
                            name: 'tx',
                            type: 'Extrinsic'
                        }
                    ],
                    type: 'TransactionValidity'
                }
            },
            version: 1
        }
    ]
};
