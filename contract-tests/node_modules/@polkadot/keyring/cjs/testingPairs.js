"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createTestPairs = createTestPairs;
const nobody_js_1 = require("./pair/nobody.js");
const testing_js_1 = require("./testing.js");
function createTestPairs(options, isDerived = true) {
    const keyring = (0, testing_js_1.createTestKeyring)(options, isDerived);
    const pairs = keyring.getPairs();
    const map = { nobody: (0, nobody_js_1.nobody)() };
    for (const p of pairs) {
        if (p.meta.name) {
            map[p.meta.name] = p;
        }
    }
    return map;
}
