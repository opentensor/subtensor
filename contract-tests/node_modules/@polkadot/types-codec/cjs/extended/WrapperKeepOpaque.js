"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.WrapperKeepOpaque = void 0;
const util_1 = require("@polkadot/util");
const Raw_js_1 = require("../native/Raw.js");
const index_js_1 = require("../utils/index.js");
const Bytes_js_1 = require("./Bytes.js");
function decodeRaw(registry, typeName, value) {
    const Type = (0, index_js_1.typeToConstructor)(registry, typeName);
    if ((0, util_1.isU8a)(value) || (0, util_1.isHex)(value)) {
        try {
            const [, u8a] = (0, util_1.isHex)(value)
                ? [0, (0, util_1.u8aToU8a)(value)]
                : (value instanceof Raw_js_1.Raw)
                    ? [0, value.subarray()]
                    : (0, util_1.compactStripLength)(value);
            return [Type, new Type(registry, u8a), value];
        }
        catch {
            return [Type, null, value];
        }
    }
    const instance = new Type(registry, value);
    return [Type, instance, (0, util_1.compactAddLength)(instance.toU8a())];
}
class WrapperKeepOpaque extends Bytes_js_1.Bytes {
    #Type;
    #decoded;
    #opaqueName;
    constructor(registry, typeName, value, { opaqueName = 'WrapperKeepOpaque' } = {}) {
        const [Type, decoded, u8a] = decodeRaw(registry, typeName, value);
        super(registry, u8a);
        this.#Type = Type;
        this.#decoded = decoded;
        this.#opaqueName = opaqueName;
    }
    static with(Type) {
        return class extends WrapperKeepOpaque {
            constructor(registry, value) {
                super(registry, Type, value);
            }
        };
    }
    /**
     * @description Checks if the wrapper is decodable
     */
    get isDecoded() {
        return !!this.#decoded;
    }
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect() {
        return this.#decoded
            ? {
                inner: [this.#decoded.inspect()],
                outer: [(0, util_1.compactToU8a)(this.length)]
            }
            : {
                outer: [(0, util_1.compactToU8a)(this.length), this.toU8a(true)]
            };
    }
    /**
     * @description Converts the Object to to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    toHuman(isExtended, disableAscii) {
        return this.#decoded
            ? this.#decoded.toHuman(isExtended, disableAscii)
            : super.toHuman(isExtended, disableAscii);
    }
    /**
     * @description Converts the value in a best-fit primitive form
     */
    toPrimitive(disableAscii) {
        return this.#decoded
            ? this.#decoded.toPrimitive(disableAscii)
            : super.toPrimitive(disableAscii);
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return `${this.#opaqueName}<${this.registry.getClassName(this.#Type) || (this.#decoded ? this.#decoded.toRawType() : new this.#Type(this.registry).toRawType())}>`;
    }
    /**
     * @description Converts the Object to to a string (either decoded or bytes)
     */
    toString() {
        return this.#decoded
            ? this.#decoded.toString()
            : super.toString();
    }
    /**
     * @description Returns the decoded that the WrapperKeepOpaque represents (if available), throws if non-decodable
     */
    unwrap() {
        if (!this.#decoded) {
            throw new Error(`${this.#opaqueName}: unwrapping an undecodable value`);
        }
        return this.#decoded;
    }
}
exports.WrapperKeepOpaque = WrapperKeepOpaque;
