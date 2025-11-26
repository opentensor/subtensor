"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const rpc_js_1 = require("./rpc.js");
exports.default = {
    rpc: rpc_js_1.rpc,
    types: {
        ExtrinsicOrHash: {
            _enum: {
                Hash: 'Hash',
                Extrinsic: 'Bytes'
            }
        },
        ExtrinsicStatus: {
            _enum: {
                Future: 'Null',
                Ready: 'Null',
                Broadcast: 'Vec<Text>',
                InBlock: 'Hash',
                Retracted: 'Hash',
                FinalityTimeout: 'Hash',
                Finalized: 'Hash',
                Usurped: 'Hash',
                Dropped: 'Null',
                Invalid: 'Null'
            }
        }
    }
};
