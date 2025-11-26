"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.rpc = void 0;
exports.rpc = {
    genSyncSpec: {
        description: 'Returns the json-serialized chainspec running the node, with a sync state.',
        endpoint: 'sync_state_genSyncSpec',
        params: [
            {
                name: 'raw',
                type: 'bool'
            }
        ],
        type: 'Json'
    }
};
