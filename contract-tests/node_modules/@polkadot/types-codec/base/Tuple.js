import { identity, isFunction, isHex, isString, isU8a, stringify, u8aConcatStrict, u8aToU8a } from '@polkadot/util';
import { AbstractArray } from '../abstract/Array.js';
import { decodeU8a, mapToTypeMap, typesToConstructors, typeToConstructor } from '../utils/index.js';
/** @internal */
function decodeTuple(registry, result, value, Classes) {
    if (Array.isArray(value)) {
        const Types = Classes[0];
        for (let i = 0, count = Types.length; i < count; i++) {
            try {
                const entry = value?.[i];
                result[i] = entry instanceof Types[i]
                    ? entry
                    : new Types[i](registry, entry);
            }
            catch (error) {
                throw new Error(`Tuple: failed on ${i}:: ${error.message}`);
            }
        }
        return [result, 0];
    }
    else if (isHex(value)) {
        return decodeU8a(registry, result, u8aToU8a(value), Classes);
    }
    else if (!value || !result.length) {
        const Types = Classes[0];
        for (let i = 0, count = Types.length; i < count; i++) {
            result[i] = new Types[i](registry);
        }
        return [result, 0];
    }
    throw new Error(`Expected array input to Tuple decoding, found ${typeof value}: ${stringify(value)}`);
}
/**
 * @name Tuple
 * @description
 * A Tuple defines an anonymous fixed-length array, where each element has its
 * own type. It extends the base JS `Array` object.
 */
export class Tuple extends AbstractArray {
    #Types;
    constructor(registry, Types, value, { definition, setDefinition = identity } = {}) {
        const Classes = definition || setDefinition(Array.isArray(Types)
            ? [typesToConstructors(registry, Types), []]
            : isFunction(Types) || isString(Types)
                ? [[typeToConstructor(registry, Types)], []]
                : mapToTypeMap(registry, Types));
        super(registry, Classes[0].length);
        this.initialU8aLength = (isU8a(value)
            ? decodeU8a(registry, this, value, Classes)
            : decodeTuple(registry, this, value, Classes))[1];
        this.#Types = Classes;
    }
    static with(Types) {
        let definition;
        // eslint-disable-next-line no-return-assign
        const setDefinition = (d) => definition = d;
        return class extends Tuple {
            constructor(registry, value) {
                super(registry, Types, value, { definition, setDefinition });
            }
        };
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
     * @description The types definition of the tuple
     */
    get Types() {
        return this.#Types[1].length
            ? this.#Types[1]
            : this.#Types[0].map((T) => new T(this.registry).toRawType());
    }
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect() {
        return {
            inner: this.inspectInner()
        };
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        const types = this.#Types[0].map((T) => this.registry.getClassName(T) || new T(this.registry).toRawType());
        return `(${types.join(',')})`;
    }
    /**
     * @description Returns the string representation of the value
     */
    toString() {
        // Overwrite the default toString representation of Array.
        return stringify(this.toJSON());
    }
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     * @param isBare true when the value has none of the type-specific prefixes (internal)
     */
    toU8a(isBare) {
        return u8aConcatStrict(this.toU8aInner(isBare));
    }
}
