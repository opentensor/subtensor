"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.runtime = void 0;
exports.runtime = {
    LocationToAccountApi: [
        {
            methods: {
                convert_location: {
                    description: 'Converts `Location` to `AccountId`',
                    params: [
                        {
                            name: 'location',
                            type: 'XcmVersionedLocation'
                        }
                    ],
                    type: 'Result<AccountId, Error>'
                }
            },
            version: 1
        }
    ]
};
