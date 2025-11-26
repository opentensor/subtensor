"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isAddress = isAddress;
const validate_js_1 = require("./validate.js");
function isAddress(address, ignoreChecksum, ss58Format) {
    try {
        return (0, validate_js_1.validateAddress)(address, ignoreChecksum, ss58Format);
    }
    catch {
        return false;
    }
}
