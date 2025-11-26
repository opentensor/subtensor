export default {
    rpc: {},
    types: {
        UncleEntryItem: {
            _enum: {
                InclusionHeight: 'BlockNumber',
                Uncle: '(Hash, Option<AccountId>)'
            }
        }
    }
};
