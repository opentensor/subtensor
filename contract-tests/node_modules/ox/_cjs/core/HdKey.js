"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.fromExtendedKey = fromExtendedKey;
exports.fromJson = fromJson;
exports.fromSeed = fromSeed;
exports.path = path;
const bip32_1 = require("@scure/bip32");
const Bytes = require("./Bytes.js");
const internal = require("./internal/hdKey.js");
function fromExtendedKey(extendedKey) {
    const key = bip32_1.HDKey.fromExtendedKey(extendedKey);
    return internal.fromScure(key);
}
function fromJson(json) {
    return internal.fromScure(bip32_1.HDKey.fromJSON(json));
}
function fromSeed(seed, options = {}) {
    const { versions } = options;
    const key = bip32_1.HDKey.fromMasterSeed(Bytes.from(seed), versions);
    return internal.fromScure(key);
}
function path(options = {}) {
    const { account = 0, change = 0, index = 0 } = options;
    return `m/44'/60'/${account}'/${change}/${index}`;
}
//# sourceMappingURL=HdKey.js.map