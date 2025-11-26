"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sshash = sshash;
const util_1 = require("@polkadot/util");
const asU8a_js_1 = require("../blake2/asU8a.js");
const SS58_PREFIX = (0, util_1.stringToU8a)('SS58PRE');
function sshash(key) {
    return (0, asU8a_js_1.blake2AsU8a)((0, util_1.u8aConcat)(SS58_PREFIX, key), 512);
}
