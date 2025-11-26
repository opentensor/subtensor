import { identity, isU8a, u8aConcatStrict } from '@polkadot/util';
import { AbstractArray } from '../abstract/Array.js';
import { decodeU8aVec, typeToConstructor } from '../utils/index.js';
import { decodeVec } from './Vec.js';
/**
 * @name VecFixed
 * @description
 * This manages codec arrays of a fixed length
 */
export class VecFixed extends AbstractArray {
    #Type;
    constructor(registry, Type, length, value = [], { definition, setDefinition = identity } = {}) {
        super(registry, length);
        this.#Type = definition || setDefinition(typeToConstructor(registry, Type));
        this.initialU8aLength = (isU8a(value)
            ? decodeU8aVec(registry, this, value, 0, this.#Type)
            : decodeVec(registry, this, value, 0, this.#Type))[1];
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
            ? u8aConcatStrict(encoded)
            : new Uint8Array([]);
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return `[${this.Type};${this.length}]`;
    }
}
