"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.keccak256 = keccak256;
exports.ripemd160 = ripemd160;
exports.sha256 = sha256;
exports.validate = validate;
const ripemd160_1 = require("@noble/hashes/ripemd160");
const sha3_1 = require("@noble/hashes/sha3");
const sha256_1 = require("@noble/hashes/sha256");
const Bytes = require("./Bytes.js");
const Hex = require("./Hex.js");
function keccak256(value, options = {}) {
    const { as = typeof value === 'string' ? 'Hex' : 'Bytes' } = options;
    const bytes = (0, sha3_1.keccak_256)(Bytes.from(value));
    if (as === 'Bytes')
        return bytes;
    return Hex.fromBytes(bytes);
}
function ripemd160(value, options = {}) {
    const { as = typeof value === 'string' ? 'Hex' : 'Bytes' } = options;
    const bytes = (0, ripemd160_1.ripemd160)(Bytes.from(value));
    if (as === 'Bytes')
        return bytes;
    return Hex.fromBytes(bytes);
}
function sha256(value, options = {}) {
    const { as = typeof value === 'string' ? 'Hex' : 'Bytes' } = options;
    const bytes = (0, sha256_1.sha256)(Bytes.from(value));
    if (as === 'Bytes')
        return bytes;
    return Hex.fromBytes(bytes);
}
function validate(value) {
    return Hex.validate(value) && Hex.size(value) === 32;
}
//# sourceMappingURL=Hash.js.map