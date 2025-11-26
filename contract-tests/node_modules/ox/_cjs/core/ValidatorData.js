"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.encode = encode;
exports.getSignPayload = getSignPayload;
const Hash = require("./Hash.js");
const Hex = require("./Hex.js");
function encode(value) {
    const { data, validator } = value;
    return Hex.concat('0x19', '0x00', validator, Hex.from(data));
}
function getSignPayload(value) {
    return Hash.keccak256(encode(value));
}
//# sourceMappingURL=ValidatorData.js.map