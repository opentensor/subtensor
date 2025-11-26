import { rpc } from './rpc.js';
import { runtime } from './runtime.js';
export default {
    rpc,
    runtime,
    types: {
        StorageKind: {
            _enum: {
                PERSISTENT: 1,
                LOCAL: 2
            }
        }
    }
};
