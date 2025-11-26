"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.runtime = void 0;
exports.runtime = {
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
