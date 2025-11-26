"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.HashMap = void 0;
const Map_js_1 = require("./Map.js");
class HashMap extends Map_js_1.CodecMap {
    static with(keyType, valType) {
        return class extends HashMap {
            constructor(registry, value) {
                super(registry, keyType, valType, value);
            }
        };
    }
}
exports.HashMap = HashMap;
