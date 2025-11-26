"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.firstObservable = firstObservable;
exports.firstMemo = firstMemo;
const rxjs_1 = require("rxjs");
const rpc_core_1 = require("@polkadot/rpc-core");
function firstObservable(obs) {
    return obs.pipe((0, rxjs_1.map)(([a]) => a));
}
function firstMemo(fn) {
    return (instanceId, api) => (0, rpc_core_1.memo)(instanceId, (...args) => firstObservable(fn(api, ...args)));
}
