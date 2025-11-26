"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.LinkageResult = exports.Linkage = void 0;
const Option_js_1 = require("../base/Option.js");
const Tuple_js_1 = require("../base/Tuple.js");
const Vec_js_1 = require("../base/Vec.js");
const Struct_js_1 = require("../native/Struct.js");
const EMPTY = new Uint8Array();
/**
 * @name Linkage
 * @description The wrapper for the result from a LinkedMap
 */
class Linkage extends Struct_js_1.Struct {
    constructor(registry, Type, value) {
        super(registry, {
            previous: Option_js_1.Option.with(Type),
            // eslint-disable-next-line sort-keys
            next: Option_js_1.Option.with(Type)
        }, value);
    }
    static withKey(Type) {
        return class extends Linkage {
            constructor(registry, value) {
                super(registry, Type, value);
            }
        };
    }
    /**
     * @description Returns the next item the Linkage is pointing to
     */
    get previous() {
        return this.get('previous');
    }
    /**
     * @description Returns the previous item the Linkage is pointing to
     */
    get next() {
        return this.get('next');
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return `Linkage<${this.next.toRawType(true)}>`;
    }
    /**
     * @description Custom toU8a which with bare mode does not return the linkage if empty
     */
    toU8a(isBare) {
        // As part of a storage query (where these appear), in the case of empty, the values
        // are NOT populated by the node - follow the same logic, leaving it empty
        return this.isEmpty
            ? EMPTY
            : super.toU8a(isBare);
    }
}
exports.Linkage = Linkage;
/**
 * @name LinkageResult
 * @description A Linkage keys/Values tuple
 */
class LinkageResult extends Tuple_js_1.Tuple {
    constructor(registry, [TypeKey, keys], [TypeValue, values]) {
        super(registry, {
            Keys: Vec_js_1.Vec.with(TypeKey),
            Values: Vec_js_1.Vec.with(TypeValue)
        }, [keys, values]);
    }
}
exports.LinkageResult = LinkageResult;
