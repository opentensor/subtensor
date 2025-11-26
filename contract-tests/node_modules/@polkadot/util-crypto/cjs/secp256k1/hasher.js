"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hasher = hasher;
const index_js_1 = require("../blake2/index.js");
const index_js_2 = require("../keccak/index.js");
function hasher(hashType, data, onlyJs) {
    return hashType === 'keccak'
        ? (0, index_js_2.keccakAsU8a)(data, undefined, onlyJs)
        : (0, index_js_1.blake2AsU8a)(data, undefined, undefined, onlyJs);
}
