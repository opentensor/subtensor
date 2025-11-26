import { Option } from '../base/Option.js';
import { Tuple } from '../base/Tuple.js';
import { Vec } from '../base/Vec.js';
import { Struct } from '../native/Struct.js';
const EMPTY = new Uint8Array();
/**
 * @name Linkage
 * @description The wrapper for the result from a LinkedMap
 */
export class Linkage extends Struct {
    constructor(registry, Type, value) {
        super(registry, {
            previous: Option.with(Type),
            // eslint-disable-next-line sort-keys
            next: Option.with(Type)
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
/**
 * @name LinkageResult
 * @description A Linkage keys/Values tuple
 */
export class LinkageResult extends Tuple {
    constructor(registry, [TypeKey, keys], [TypeValue, values]) {
        super(registry, {
            Keys: Vec.with(TypeKey),
            Values: Vec.with(TypeValue)
        }, [keys, values]);
    }
}
