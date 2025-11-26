"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.fallbackExtensions = exports.allExtensions = void 0;
exports.findUnknownExtensions = findUnknownExtensions;
exports.expandExtensionTypes = expandExtensionTypes;
const util_1 = require("@polkadot/util");
const polkadot_js_1 = require("./polkadot.js");
const shell_js_1 = require("./shell.js");
const statemint_js_1 = require("./statemint.js");
const substrate_js_1 = require("./substrate.js");
exports.allExtensions = (0, util_1.objectSpread)({}, substrate_js_1.substrate, polkadot_js_1.polkadot, shell_js_1.shell, statemint_js_1.statemint);
exports.fallbackExtensions = [
    'CheckVersion',
    'CheckGenesis',
    'CheckEra',
    'CheckNonce',
    'CheckWeight',
    'ChargeTransactionPayment',
    'CheckBlockGasLimit'
];
function findUnknownExtensions(extensions, userExtensions = {}) {
    const names = [...Object.keys(exports.allExtensions), ...Object.keys(userExtensions)];
    return extensions.filter((k) => !names.includes(k));
}
function expandExtensionTypes(extensions, type, userExtensions = {}) {
    return extensions
        // Always allow user extensions first - these should provide overrides
        .map((k) => userExtensions[k] || exports.allExtensions[k])
        .filter((info) => !!info)
        .reduce((result, info) => (0, util_1.objectSpread)(result, info[type]), {});
}
