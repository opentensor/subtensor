"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.BTreeMap = void 0;
const Map_js_1 = require("./Map.js");
class BTreeMap extends Map_js_1.CodecMap {
    static with(keyType, valType) {
        return class extends BTreeMap {
            constructor(registry, value) {
                super(registry, keyType, valType, value, 'BTreeMap');
            }
        };
    }
}
exports.BTreeMap = BTreeMap;
