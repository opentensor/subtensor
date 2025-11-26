"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.defaults = void 0;
const networks_js_1 = require("../networks.js");
exports.defaults = {
    allowedDecodedLengths: [1, 2, 4, 8, 32, 33],
    // publicKey has prefix + 2 checksum bytes, short only prefix + 1 checksum byte
    allowedEncodedLengths: [3, 4, 6, 10, 35, 36, 37, 38],
    allowedPrefix: networks_js_1.availableNetworks.map(({ prefix }) => prefix),
    prefix: 42
};
