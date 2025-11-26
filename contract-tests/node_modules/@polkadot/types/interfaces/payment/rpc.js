export const rpc = {
    queryFeeDetails: {
        // NOTE: Not deprecated (yet) in Substrate, but it is the intent to do so
        deprecated: 'Use `api.call.transactionPaymentApi.queryFeeDetails` instead',
        description: 'Query the detailed fee of a given encoded extrinsic',
        params: [
            {
                name: 'extrinsic',
                type: 'Bytes'
            },
            {
                isHistoric: true,
                isOptional: true,
                name: 'at',
                type: 'BlockHash'
            }
        ],
        type: 'FeeDetails'
    },
    queryInfo: {
        // NOTE: Not deprecated (yet) in Substrate, but it is the intent to do so
        deprecated: 'Use `api.call.transactionPaymentApi.queryInfo` instead',
        description: 'Retrieves the fee information for an encoded extrinsic',
        params: [
            {
                name: 'extrinsic',
                type: 'Bytes'
            },
            {
                isHistoric: true,
                isOptional: true,
                name: 'at',
                type: 'BlockHash'
            }
        ],
        // NOTE: Stuck on V1 forever (at least for the time being)
        type: 'RuntimeDispatchInfoV1'
    }
};
