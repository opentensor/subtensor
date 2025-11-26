import { runtime } from './runtime.js';
export default {
    rpc: {},
    runtime,
    types: {
        Mixnode: {
            externalAddresses: 'Vec<Bytes>',
            kxPublic: '[u8; 32]',
            peerId: '[u8; 32]'
        },
        MixnodesErr: {
            _enum: {
                InsufficientRegistrations: {
                    min: 'u32',
                    num: 'u32'
                }
            }
        },
        SessionPhase: {
            _enum: ['CoverToCurrent', 'RequestsToCurrent', 'CoverToPrev', 'DisconnectFromPrev']
        },
        SessionStatus: {
            currentIndex: 'u32',
            phase: 'SessionPhase'
        }
    }
};
