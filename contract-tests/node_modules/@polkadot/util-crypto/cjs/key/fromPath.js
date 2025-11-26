"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.keyFromPath = keyFromPath;
const hdkdEcdsa_js_1 = require("./hdkdEcdsa.js");
const hdkdEd25519_js_1 = require("./hdkdEd25519.js");
const hdkdSr25519_js_1 = require("./hdkdSr25519.js");
const generators = {
    ecdsa: hdkdEcdsa_js_1.keyHdkdEcdsa,
    ed25519: hdkdEd25519_js_1.keyHdkdEd25519,
    // FIXME This is Substrate-compatible, not Ethereum-compatible
    ethereum: hdkdEcdsa_js_1.keyHdkdEcdsa,
    sr25519: hdkdSr25519_js_1.keyHdkdSr25519
};
function keyFromPath(pair, path, type) {
    const keyHdkd = generators[type];
    let result = pair;
    for (const junction of path) {
        result = keyHdkd(result, junction);
    }
    return result;
}
