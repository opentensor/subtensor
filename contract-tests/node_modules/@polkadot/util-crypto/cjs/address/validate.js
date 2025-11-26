"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateAddress = validateAddress;
const decode_js_1 = require("./decode.js");
function validateAddress(encoded, ignoreChecksum, ss58Format) {
    return !!(0, decode_js_1.decodeAddress)(encoded, ignoreChecksum, ss58Format);
}
