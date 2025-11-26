"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.rpc = void 0;
exports.rpc = {
    localStorageClear: {
        description: 'Clear offchain local storage under given key and prefix',
        isUnsafe: true,
        params: [
            {
                name: 'kind',
                type: 'StorageKind'
            },
            {
                name: 'key',
                type: 'Bytes'
            }
        ],
        type: 'Null'
    },
    localStorageGet: {
        description: 'Get offchain local storage under given key and prefix',
        isUnsafe: true,
        params: [
            {
                name: 'kind',
                type: 'StorageKind'
            },
            {
                name: 'key',
                type: 'Bytes'
            }
        ],
        type: 'Option<Bytes>'
    },
    localStorageSet: {
        description: 'Set offchain local storage under given key and prefix',
        isUnsafe: true,
        params: [
            {
                name: 'kind',
                type: 'StorageKind'
            },
            {
                name: 'key',
                type: 'Bytes'
            },
            {
                name: 'value',
                type: 'Bytes'
            }
        ],
        type: 'Null'
    }
};
