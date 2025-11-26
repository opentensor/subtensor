export default {
    rpc: {},
    types: {
        ActiveGilt: {
            proportion: 'Perquintill',
            amount: 'Balance',
            who: 'AccountId',
            expiry: 'BlockNumber'
        },
        ActiveGiltsTotal: {
            frozen: 'Balance',
            proportion: 'Perquintill',
            index: 'ActiveIndex',
            target: 'Perquintill'
        },
        ActiveIndex: 'u32',
        GiltBid: {
            amount: 'Balance',
            who: 'AccountId'
        }
    }
};
