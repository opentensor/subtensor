"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createStore = createStore;
exports.from = from;
function createStore(options = {}) {
    let id = options.id ?? 0;
    return {
        prepare(options) {
            return from({
                id: id++,
                ...options,
            });
        },
        get id() {
            return id;
        },
    };
}
function from(options) {
    return {
        ...options,
        jsonrpc: '2.0',
    };
}
//# sourceMappingURL=RpcRequest.js.map