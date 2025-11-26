"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getStorage = getStorage;
const substrate_js_1 = require("./substrate.js");
/** @internal */
function getStorage(registry) {
    const storage = {};
    const entries = Object.entries(substrate_js_1.substrate);
    for (let e = 0, count = entries.length; e < count; e++) {
        storage[entries[e][0]] = entries[e][1](registry);
    }
    return { substrate: storage };
}
