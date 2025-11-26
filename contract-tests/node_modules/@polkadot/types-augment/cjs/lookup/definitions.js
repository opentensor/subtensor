"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const tslib_1 = require("tslib");
const util_1 = require("@polkadot/util");
const assetHubKusama_js_1 = tslib_1.__importDefault(require("./assetHubKusama.js"));
const assetHubPolkadot_js_1 = tslib_1.__importDefault(require("./assetHubPolkadot.js"));
const kusama_js_1 = tslib_1.__importDefault(require("./kusama.js"));
const polkadot_js_1 = tslib_1.__importDefault(require("./polkadot.js"));
const substrate_js_1 = tslib_1.__importDefault(require("./substrate.js"));
exports.default = {
    rpc: {},
    // Not 100% sure it is relevant, however the order here is the same
    // as exposed in the typegen lookup order
    types: (0, util_1.objectSpread)({}, substrate_js_1.default, polkadot_js_1.default, kusama_js_1.default, assetHubPolkadot_js_1.default, assetHubKusama_js_1.default)
};
