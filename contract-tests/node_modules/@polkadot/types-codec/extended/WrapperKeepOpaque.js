import { compactAddLength, compactStripLength, compactToU8a, isHex, isU8a, u8aToU8a } from '@polkadot/util';
import { Raw } from '../native/Raw.js';
import { typeToConstructor } from '../utils/index.js';
import { Bytes } from './Bytes.js';
function decodeRaw(registry, typeName, value) {
    const Type = typeToConstructor(registry, typeName);
    if (isU8a(value) || isHex(value)) {
        try {
            const [, u8a] = isHex(value)
                ? [0, u8aToU8a(value)]
                : (value instanceof Raw)
                    ? [0, value.subarray()]
                    : compactStripLength(value);
            return [Type, new Type(registry, u8a), value];
        }
        catch {
            return [Type, null, value];
        }
    }
    const instance = new Type(registry, value);
    return [Type, instance, compactAddLength(instance.toU8a())];
}
export class WrapperKeepOpaque extends Bytes {
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
                outer: [compactToU8a(this.length)]
            }
            : {
                outer: [compactToU8a(this.length), this.toU8a(true)]
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
