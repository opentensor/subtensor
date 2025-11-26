"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.runtime = void 0;
exports.runtime = {
    SessionKeys: [
        {
            methods: {
                decode_session_keys: {
                    description: 'Decode the given public session keys.',
                    params: [
                        {
                            name: 'encoded',
                            type: 'Bytes'
                        }
                    ],
                    type: 'Option<Vec<(Bytes, KeyTypeId)>>'
                },
                generate_session_keys: {
                    description: 'Generate a set of session keys with optionally using the given seed.',
                    params: [
                        {
                            name: 'seed',
                            type: 'Option<Bytes>'
                        }
                    ],
                    type: 'Bytes'
                }
            },
            version: 1
        }
    ]
};
