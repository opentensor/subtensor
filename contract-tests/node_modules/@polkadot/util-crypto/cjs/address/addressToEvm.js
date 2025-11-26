"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.addressToEvm = addressToEvm;
const decode_js_1 = require("./decode.js");
/**
 * @name addressToEvm
 * @summary Converts an SS58 address to its corresponding EVM address.
 */
function addressToEvm(address, ignoreChecksum) {
    return (0, decode_js_1.decodeAddress)(address, ignoreChecksum).subarray(0, 20);
}
