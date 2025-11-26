"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const tslib_1 = require("tslib");
const util_1 = require("@polkadot/util");
const defs = tslib_1.__importStar(require("./definitions.js"));
const jsonrpc = {};
Object.keys(defs).forEach((s) => Object.entries(defs[s].rpc || {}).forEach(([method, def]) => {
    // allow for section overrides
    const section = def.aliasSection || s;
    if (!jsonrpc[section]) {
        jsonrpc[section] = {};
    }
    jsonrpc[section][method] = (0, util_1.objectSpread)({}, def, {
        isSubscription: !!def.pubsub,
        jsonrpc: `${section}_${method}`,
        method,
        section
    });
}));
exports.default = jsonrpc;
