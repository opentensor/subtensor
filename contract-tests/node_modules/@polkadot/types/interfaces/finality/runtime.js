const finalityV1 = {
    methods: {
        best_finalized: {
            description: 'Returns number and hash of the best finalized header known to the bridge module.',
            params: [],
            type: '(BlockNumber, Hash)'
        }
    },
    version: 1
};
export const runtime = {
    KusamaFinalityApi: [finalityV1],
    PolkadotFinalityApi: [finalityV1],
    RococoFinalityApi: [finalityV1],
    WestendFinalityApi: [finalityV1]
};
