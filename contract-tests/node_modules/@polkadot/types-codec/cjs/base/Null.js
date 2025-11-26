"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Null = void 0;
const util_1 = require("@polkadot/util");
/**
 * @name Null
 * @description
 * Implements a type that does not contain anything (apart from `null`)
 */
class Null {
    encodedLength = 0;
    isEmpty = true;
    registry;
    createdAtHash;
    initialU8aLength = 0;
    isStorageFallback;
    constructor(registry) {
        this.registry = registry;
    }
    /**
     * @description returns a hash of the contents
     */
    get hash() {
        throw new Error('.hash is not implemented on Null');
    }
    /**
     * @description Compares the value of the input to see if there is a match
     */
    eq(other) {
        return other instanceof Null || (0, util_1.isNull)(other);
    }
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect() {
        return {};
    }
    /**
     * @description Returns a hex string representation of the value
     */
    toHex() {
        return '0x';
    }
    /**
     * @description Converts the Object to to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    toHuman() {
        return this.toJSON();
    }
    /**
     * @description Converts the Object to JSON, typically used for RPC transfers
     */
    toJSON() {
        return null;
    }
    /**
     * @description Converts the value in a best-fit primitive form
     */
    toPrimitive() {
        return null;
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return 'Null';
    }
    /**
     * @description Returns the string representation of the value
     */
    toString() {
        return '';
    }
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     */
    toU8a(_isBare) {
        return new Uint8Array();
    }
}
exports.Null = Null;
