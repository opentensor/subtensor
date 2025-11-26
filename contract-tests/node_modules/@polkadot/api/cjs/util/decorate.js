"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.decorateDeriveSections = decorateDeriveSections;
const api_derive_1 = require("@polkadot/api-derive");
/**
 * This is a section decorator which keeps all type information.
 */
function decorateDeriveSections(decorateMethod, derives) {
    const getKeys = (s) => Object.keys(derives[s]);
    const creator = (s, m) => decorateMethod(derives[s][m]);
    const result = {};
    const names = Object.keys(derives);
    for (let i = 0, count = names.length; i < count; i++) {
        (0, api_derive_1.lazyDeriveSection)(result, names[i], getKeys, creator);
    }
    return result;
}
