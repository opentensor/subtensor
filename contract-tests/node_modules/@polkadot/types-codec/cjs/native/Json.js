"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Json = void 0;
const util_1 = require("@polkadot/util");
const index_js_1 = require("../utils/index.js");
/** @internal */
function decodeJson(value) {
    return Object.entries(value || {});
}
/**
 * @name Json
 * @description
 * Wraps the a JSON structure retrieve via RPC. It extends the standard JS Map with. While it
 * implements a Codec, it is limited in that it can only be used with input objects via RPC,
 * i.e. no hex decoding. Unlike a struct, this waps a JSON object with unknown keys
 * @noInheritDoc
 */
class Json extends Map {
    registry;
    createdAtHash;
    initialU8aLength;
    isStorageFallback;
    constructor(registry, value) {
        const decoded = decodeJson(value);
        super(decoded);
        this.registry = registry;
        (0, util_1.objectProperties)(this, decoded.map(([k]) => k), (k) => this.get(k));
    }
    /**
     * @description Always 0, never encodes as a Uint8Array
     */
    get encodedLength() {
        return 0 | 0;
    }
    /**
     * @description returns a hash of the contents
     */
    get hash() {
        return this.registry.hash(this.toU8a());
    }
    /**
     * @description Checks if the value is an empty value
     */
    get isEmpty() {
        return [...this.keys()].length === 0;
    }
    /**
     * @description Compares the value of the input to see if there is a match
     */
    eq(other) {
        return (0, index_js_1.compareMap)(this, other);
    }
    /**
     * @description Returns a typed value from the internal map
     */
    getT(key) {
        return this.get(key);
    }
    /**
     * @description Unimplemented, will throw
     */
    inspect() {
        throw new Error('Unimplemented');
    }
    /**
     * @description Unimplemented, will throw
     */
    toHex() {
        throw new Error('Unimplemented');
    }
    /**
     * @description Converts the Object to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    toHuman() {
        return [...this.entries()].reduce((json, [key, value]) => {
            json[key] = (0, util_1.isFunction)(value?.toHuman)
                ? value.toHuman()
                : value;
            return json;
        }, {});
    }
    /**
     * @description Converts the Object to JSON, typically used for RPC transfers
     */
    toJSON() {
        return [...this.entries()].reduce((json, [key, value]) => {
            json[key] = value;
            return json;
        }, {});
    }
    /**
     * @description Converts the value in a best-fit primitive form
     */
    toPrimitive(disableAscii) {
        return [...this.entries()].reduce((json, [key, value]) => {
            json[key] = (0, util_1.isFunction)(value.toPrimitive)
                ? value.toPrimitive(disableAscii)
                : value;
            return json;
        }, {});
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return 'Json';
    }
    /**
     * @description Returns the string representation of the value
     */
    toString() {
        return (0, util_1.stringify)(this.toJSON());
    }
    /**
     * @description Unimplemented, will throw
     */
    toU8a(_isBare) {
        throw new Error('Unimplemented');
    }
}
exports.Json = Json;
