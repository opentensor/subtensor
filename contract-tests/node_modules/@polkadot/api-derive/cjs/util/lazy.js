"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.lazyDeriveSection = lazyDeriveSection;
const util_1 = require("@polkadot/util");
function lazyDeriveSection(result, section, getKeys, creator) {
    (0, util_1.lazyMethod)(result, section, () => (0, util_1.lazyMethods)({}, getKeys(section), (method) => creator(section, method)));
}
