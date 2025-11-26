"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.runtime = void 0;
exports.runtime = {
    AuraApi: [
        {
            methods: {
                authorities: {
                    description: 'Return the current set of authorities.',
                    params: [],
                    type: 'Vec<AuthorityId>'
                },
                slot_duration: {
                    description: 'Returns the slot duration for Aura.',
                    params: [],
                    type: 'SlotDuration'
                }
            },
            version: 1
        }
    ]
};
