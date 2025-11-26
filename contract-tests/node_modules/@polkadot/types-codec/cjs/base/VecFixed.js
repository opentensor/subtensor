"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.VecFixed = void 0;
const util_1 = require("@polkadot/util");
const Array_js_1 = require("../abstract/Array.js");
const index_js_1 = require("../utils/index.js");
const Vec_js_1 = require("./Vec.js");
/**
 * @name VecFixed
 * @description
 * This manages codec arrays of a fixed length
 */
class VecFixed extends Array_js_1.AbstractArray {
    #Type;
    constructor(registry, Type, length, value = [], { definition, setDefinition = util_1.identity } = {}) {
        super(registry, length);
        this.#Type = definition || setDefinition((0, index_js_1.typeToConstructor)(registry, Type));
        this.initialU8aLength = ((0, util_1.isU8a)(value)
            ? (0, index_js_1.decodeU8aVec)(registry, this, value, 0, this.#Type)
            : (0, Vec_js_1.decodeVec)(registry, this, value, 0, this.#Type))[1];
    }
    static with(Type, length) {
        let definition;
        // eslint-disable-next-line no-return-assign
        const setDefinition = (d) => (definition = d);
        return class extends VecFixed {
            constructor(registry, value) {
                super(registry, Type, length, value, { definition, setDefinition });
            }
        };
    }
    /**
     * @description The type for the items
     */
    get Type() {
        return new this.#Type(this.registry).toRawType();
    }
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength() {
        let total = 0;
        for (let i = 0, count = this.length; i < count; i++) {
            total += this[i].encodedLength;
        }
        return total;
    }
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect() {
        return {
            inner: this.inspectInner()
        };
    }
    toU8a() {
        // we override, we don't add the length prefix for ourselves, and at the same time we
        // ignore isBare on entries, since they should be properly encoded at all times
        const encoded = this.toU8aInner();
        return encoded.length
            ? (0, util_1.u8aConcatStrict)(encoded)
            : new Uint8Array([]);
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return `[${this.Type};${this.length}]`;
    }
}
exports.VecFixed = VecFixed;
