"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const rpc_js_1 = require("./rpc.js");
exports.default = {
    rpc: rpc_js_1.rpc,
    types: {
        RpcMethods: {
            version: 'u32',
            methods: 'Vec<Text>'
        }
    }
};
