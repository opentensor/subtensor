export const runtime = {
    AuthorityDiscoveryApi: [
        {
            methods: {
                authorities: {
                    description: 'Retrieve authority identifiers of the current and next authority set.',
                    params: [],
                    type: 'Vec<AuthorityId>'
                }
            },
            version: 1
        }
    ]
};
