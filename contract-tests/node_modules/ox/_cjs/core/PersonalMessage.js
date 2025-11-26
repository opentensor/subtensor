"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.encode = encode;
exports.getSignPayload = getSignPayload;
const Hash = require("./Hash.js");
const Hex = require("./Hex.js");
function encode(data) {
    const message = Hex.from(data);
    return Hex.concat('0x19', Hex.fromString('Ethereum Signed Message:\n' + Hex.size(message)), message);
}
function getSignPayload(data) {
    return Hash.keccak256(encode(data));
}
//# sourceMappingURL=PersonalMessage.js.map