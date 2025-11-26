export default {
    rpc: {},
    types: {
        AccountStatus: {
            validity: 'AccountValidity',
            freeBalance: 'Balance',
            lockedBalance: 'Balance',
            signature: 'Vec<u8>',
            vat: 'Permill'
        },
        AccountValidity: {
            _enum: ['Invalid', 'Initiated', 'Pending', 'ValidLow', 'ValidHigh', 'Completed']
        }
    }
};
