export default {
    rpc: {},
    types: {
        ProxyDefinition: {
            delegate: 'AccountId',
            proxyType: 'ProxyType',
            delay: 'BlockNumber'
        },
        ProxyType: {
            _enum: ['Any', 'NonTransfer', 'Governance', 'Staking']
        },
        ProxyAnnouncement: {
            real: 'AccountId',
            callHash: 'Hash',
            height: 'BlockNumber'
        }
    }
};
