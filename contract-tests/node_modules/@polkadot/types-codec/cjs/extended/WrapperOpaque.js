"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.WrapperOpaque = void 0;
const WrapperKeepOpaque_js_1 = require("./WrapperKeepOpaque.js");
class WrapperOpaque extends WrapperKeepOpaque_js_1.WrapperKeepOpaque {
    constructor(registry, typeName, value) {
        super(registry, typeName, value, { opaqueName: 'WrapperOpaque' });
    }
    static with(Type) {
        return class extends WrapperOpaque {
            constructor(registry, value) {
                super(registry, Type, value);
            }
        };
    }
    /**
     * @description The inner value for this wrapper, in all cases it _should_ be decodable (unlike KeepOpaque)
     */
    get inner() {
        return this.unwrap();
    }
}
exports.WrapperOpaque = WrapperOpaque;
