"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sortAddresses = sortAddresses;
const util_1 = require("@polkadot/util");
const encode_js_1 = require("./encode.js");
const util_js_1 = require("./util.js");
function sortAddresses(addresses, ss58Format) {
    const u8aToAddress = (u8a) => (0, encode_js_1.encodeAddress)(u8a, ss58Format);
    return (0, util_1.u8aSorted)(addresses.map(util_js_1.addressToU8a)).map(u8aToAddress);
}
