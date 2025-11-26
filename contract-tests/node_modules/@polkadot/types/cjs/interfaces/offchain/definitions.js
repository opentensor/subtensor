"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const rpc_js_1 = require("./rpc.js");
const runtime_js_1 = require("./runtime.js");
exports.default = {
    rpc: rpc_js_1.rpc,
    runtime: runtime_js_1.runtime,
    types: {
        StorageKind: {
            _enum: {
                PERSISTENT: 1,
                LOCAL: 2
            }
        }
    }
};
