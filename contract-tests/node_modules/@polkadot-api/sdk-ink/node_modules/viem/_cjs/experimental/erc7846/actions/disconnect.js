"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.disconnect = disconnect;
async function disconnect(client) {
    return await client.request({ method: 'wallet_disconnect' }, { dedupe: true, retryCount: 0 });
}
//# sourceMappingURL=disconnect.js.map