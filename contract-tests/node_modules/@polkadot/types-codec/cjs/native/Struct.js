"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Struct = void 0;
const util_1 = require("@polkadot/util");
const index_js_1 = require("../utils/index.js");
function noopSetDefinition(d) {
    return d;
}
/** @internal */
function decodeStructFromObject(registry, [Types, keys], value, jsonMap) {
    let jsonObj;
    const typeofArray = Array.isArray(value);
    const typeofMap = value instanceof Map;
    const count = keys.length;
    if (!typeofArray && !typeofMap && !(0, util_1.isObject)(value)) {
        throw new Error(`Struct: Cannot decode value ${(0, util_1.stringify)(value)} (typeof ${typeof value}), expected an input object, map or array`);
    }
    else if (typeofArray && value.length !== count) {
        throw new Error(`Struct: Unable to map ${(0, util_1.stringify)(value)} array to object with known keys ${keys.join(', ')}`);
    }
    const raw = new Array(count);
    for (let i = 0; i < count; i++) {
        const key = keys[i];
        const jsonKey = jsonMap.get(key) || key;
        const Type = Types[i];
        let assign;
        try {
            if (typeofArray) {
                assign = value[i];
            }
            else if (typeofMap) {
                assign = jsonKey && value.get(jsonKey);
            }
            else {
                assign = jsonKey && Object.prototype.hasOwnProperty.call(value, jsonKey) ? value[jsonKey] : undefined;
                if ((0, util_1.isUndefined)(assign)) {
                    if ((0, util_1.isUndefined)(jsonObj)) {
                        const entries = Object.entries(value);
                        jsonObj = {};
                        for (let e = 0, ecount = entries.length; e < ecount; e++) {
                            if (Object.prototype.hasOwnProperty.call(value, entries[e][0])) {
                                jsonObj[(0, util_1.stringCamelCase)(entries[e][0])] = entries[e][1];
                            }
                        }
                    }
                    assign = jsonKey && Object.prototype.hasOwnProperty.call(jsonObj, jsonKey) ? jsonObj[jsonKey] : undefined;
                }
            }
            raw[i] = [
                key,
                assign instanceof Type
                    ? assign
                    : new Type(registry, assign)
            ];
        }
        catch (error) {
            let type = Type.name;
            try {
                type = new Type(registry).toRawType();
            }
            catch {
                // ignore
            }
            throw new Error(`Struct: failed on ${jsonKey}: ${type}:: ${error.message}`);
        }
    }
    return [raw, 0];
}
/**
 * @name Struct
 * @description
 * A Struct defines an Object with key-value pairs - where the values are Codec values. It removes
 * a lot of repetition from the actual coding, define a structure type, pass it the key/Codec
 * values in the constructor and it manages the decoding. It is important that the constructor
 * values matches 100% to the order in th Rust code, i.e. don't go crazy and make it alphabetical,
 * it needs to decoded in the specific defined order.
 * @noInheritDoc
 */
class Struct extends Map {
    registry;
    createdAtHash;
    initialU8aLength;
    isStorageFallback;
    #jsonMap;
    #Types;
    constructor(registry, Types, value, jsonMap = new Map(), { definition, setDefinition = noopSetDefinition } = {}) {
        const typeMap = definition || setDefinition((0, index_js_1.mapToTypeMap)(registry, Types));
        const [decoded, decodedLength] = (0, util_1.isU8a)(value) || (0, util_1.isHex)(value)
            ? (0, index_js_1.decodeU8aStruct)(registry, new Array(typeMap[0].length), (0, util_1.u8aToU8a)(value), typeMap)
            : value instanceof Struct
                ? [value, 0]
                : decodeStructFromObject(registry, typeMap, value || {}, jsonMap);
        super(decoded);
        this.initialU8aLength = decodedLength;
        this.registry = registry;
        this.#jsonMap = jsonMap;
        this.#Types = typeMap;
    }
    static with(Types, jsonMap) {
        let definition;
        // eslint-disable-next-line no-return-assign
        const setDefinition = (d) => definition = d;
        return class extends Struct {
            static {
                const keys = Object.keys(Types);
                (0, util_1.objectProperties)(this.prototype, keys, (k, _, self) => self.get(k));
            }
            constructor(registry, value) {
                super(registry, Types, value, jsonMap, { definition, setDefinition });
            }
        };
    }
    /**
     * @description The available keys for this struct
     */
    get defKeys() {
        return this.#Types[1];
    }
    /**
     * @description Checks if the value is an empty value '{}'
     */
    get isEmpty() {
        return [...this.keys()].length === 0;
    }
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength() {
        let total = 0;
        for (const v of this.values()) {
            total += v.encodedLength;
        }
        return total;
    }
    /**
     * @description returns a hash of the contents
     */
    get hash() {
        return this.registry.hash(this.toU8a());
    }
    /**
     * @description Returns the Type description of the structure
     */
    get Type() {
        const result = {};
        const [Types, keys] = this.#Types;
        for (let i = 0, count = keys.length; i < count; i++) {
            result[keys[i]] = new Types[i](this.registry).toRawType();
        }
        return result;
    }
    /**
     * @description Compares the value of the input to see if there is a match
     */
    eq(other) {
        return (0, index_js_1.compareMap)(this, other);
    }
    /**
     * @description Returns a specific names entry in the structure
     * @param key The name of the entry to retrieve
     */
    get(key) {
        return super.get(key);
    }
    /**
     * @description Returns the values of a member at a specific index (Rather use get(name) for performance)
     */
    getAtIndex(index) {
        return this.toArray()[index];
    }
    /**
     * @description Returns the a types value by name
     */
    getT(key) {
        return super.get(key);
    }
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect(isBare) {
        const inner = [];
        for (const [k, v] of this.entries()) {
            inner.push({
                ...v.inspect(!isBare || (0, util_1.isBoolean)(isBare)
                    ? isBare
                    : isBare[k]),
                name: (0, util_1.stringCamelCase)(k)
            });
        }
        return {
            inner
        };
    }
    /**
     * @description Converts the Object to an standard JavaScript Array
     */
    toArray() {
        return [...this.values()];
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
    toHuman(isExtended, disableAscii) {
        const json = {};
        for (const [k, v] of this.entries()) {
            json[k] = v.toHuman(isExtended, disableAscii);
        }
        return json;
    }
    /**
     * @description Converts the Object to JSON, typically used for RPC transfers
     */
    toJSON() {
        const json = {};
        for (const [k, v] of this.entries()) {
            // Here we pull out the entry against the JSON mapping (if supplied)
            // since this representation goes over RPC and needs to be correct
            json[(this.#jsonMap.get(k) || k)] = v.toJSON();
        }
        return json;
    }
    /**
     * @description Converts the value in a best-fit primitive form
     */
    toPrimitive(disableAscii) {
        const json = {};
        for (const [k, v] of this.entries()) {
            json[k] = v.toPrimitive(disableAscii);
        }
        return json;
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return (0, util_1.stringify)((0, index_js_1.typesToMap)(this.registry, this.#Types));
    }
    /**
     * @description Returns the string representation of the value
     */
    toString() {
        return (0, util_1.stringify)(this.toJSON());
    }
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     * @param isBare true when the value has none of the type-specific prefixes (internal)
     */
    toU8a(isBare) {
        const encoded = [];
        for (const [k, v] of this.entries()) {
            encoded.push(v.toU8a(!isBare || (0, util_1.isBoolean)(isBare)
                ? isBare
                : isBare[k]));
        }
        return (0, util_1.u8aConcatStrict)(encoded);
    }
}
exports.Struct = Struct;
