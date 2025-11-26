export default {
    rpc: {},
    types: {
        CallIndex: '(u8, u8)',
        LotteryConfig: {
            price: 'Balance',
            start: 'BlockNumber',
            length: 'BlockNumber',
            delay: 'BlockNumber',
            repeat: 'bool'
        }
    }
};
