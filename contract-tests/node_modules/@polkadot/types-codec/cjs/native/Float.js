"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Float = void 0;
const util_1 = require("@polkadot/util");
/**
 * @name Float
 * @description
 * A Codec wrapper for F32 & F64 values. You generally don't want to be using
 * f32/f64 in your runtime, operations on fixed points numbers are preferable. This class
 * was explicitly added since scale-codec has a flag that enables this and it is available
 * in some eth_* RPCs
 */
class Float extends Number {
    encodedLength;
    registry;
    createdAtHash;
    initialU8aLength;
    isStorageFallback;
    #bitLength;
    constructor(registry, value, { bitLength = 32 } = {}) {
        super((0, util_1.isU8a)(value) || (0, util_1.isHex)(value)
            ? value.length === 0
                ? 0
                : (0, util_1.u8aToFloat)((0, util_1.u8aToU8a)(value), { bitLength })
            : (value || 0));
        this.#bitLength = bitLength;
        this.encodedLength = bitLength / 8;
        this.initialU8aLength = this.encodedLength;
        this.registry = registry;
    }
    static with(bitLength) {
        return class extends Float {
            constructor(registry, value) {
                super(registry, value, { bitLength });
            }
        };
    }
    /**
     * @description returns a hash of the contents
     */
    get hash() {
        return this.registry.hash(this.toU8a());
    }
    /**
     * @description Returns true if the type wraps an empty/default all-0 value
     */
    get isEmpty() {
        return this.valueOf() === 0;
    }
    /**
     * @description Compares the value of the input to see if there is a match
     */
    eq(other) {
        return this.valueOf() === Number(other);
    }
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect() {
        return {
            outer: [this.toU8a()]
        };
    }
    /**
     * @description Returns a hex string representation of the value
     */
    toHex() {
        return (0, util_1.u8aToHex)(this.toU8a());
    }
    /**
     * @description Converts the Object to to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    toHuman() {
        return this.toString();
    }
    /**
     * @description Converts the Object to JSON, typically used for RPC transfers
     */
    toJSON() {
        // Not sure if this is actually a hex or a string value
        // (would need to check against RPCs to see the result here)
        return this.toHex();
    }
    /**
     * @description Returns the number representation (Same as valueOf)
     */
    toNumber() {
        return this.valueOf();
    }
    /**
     * @description Converts the value in a best-fit primitive form
     */
    toPrimitive() {
        return this.toNumber();
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return `f${this.#bitLength}`;
    }
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     */
    toU8a(_isBare) {
        return (0, util_1.floatToU8a)(this, {
            bitLength: this.#bitLength
        });
    }
}
exports.Float = Float;
