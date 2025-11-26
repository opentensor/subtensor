"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = {
    rpc: {},
    types: {
        AuthIndex: 'u32',
        AuthoritySignature: 'Signature',
        Heartbeat: {
            blockNumber: 'BlockNumber',
            networkState: 'OpaqueNetworkState',
            sessionIndex: 'SessionIndex',
            authorityIndex: 'AuthIndex',
            validatorsLen: 'u32'
        },
        HeartbeatTo244: {
            blockNumber: 'BlockNumber',
            networkState: 'OpaqueNetworkState',
            sessionIndex: 'SessionIndex',
            authorityIndex: 'AuthIndex'
        },
        OpaqueMultiaddr: 'Opaque<Bytes>',
        OpaquePeerId: 'Opaque<Bytes>',
        OpaqueNetworkState: {
            peerId: 'OpaquePeerId',
            externalAddresses: 'Vec<OpaqueMultiaddr>'
        }
    }
};
