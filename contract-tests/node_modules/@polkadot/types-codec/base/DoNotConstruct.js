/**
 * @name DoNotConstruct
 * @description
 * An unknown type that fails on construction with the type info
 */
export class DoNotConstruct {
    registry;
    createdAtHash;
    isStorageFallback;
    #neverError;
    constructor(registry, typeName = 'DoNotConstruct') {
        this.registry = registry;
        this.#neverError = new Error(`DoNotConstruct: Cannot construct unknown type ${typeName}`);
        throw this.#neverError;
    }
    static with(typeName) {
        return class extends DoNotConstruct {
            constructor(registry) {
                super(registry, typeName);
            }
        };
    }
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength() {
        throw this.#neverError;
    }
    /**
     * @description returns a hash of the contents
     */
    get hash() {
        throw this.#neverError;
    }
    /**
     * @description Checks if the value is an empty value (always true)
     */
    get isEmpty() {
        throw this.#neverError;
    }
    /**
     * @description Unimplemented
     */
    eq() {
        throw this.#neverError;
    }
    /**
     * @description Unimplemented
     */
    inspect() {
        throw this.#neverError;
    }
    /**
     * @description Unimplemented
     */
    toHex() {
        throw this.#neverError;
    }
    /**
     * @description Unimplemented
     */
    toHuman() {
        throw this.#neverError;
    }
    /**
     * @description Unimplemented
     */
    toJSON() {
        throw this.#neverError;
    }
    /**
     * @description Unimplemented
     */
    toPrimitive() {
        throw this.#neverError;
    }
    /**
     * @description Unimplemented
     */
    toRawType() {
        throw this.#neverError;
    }
    /**
     * @description Unimplemented
     */
    toString() {
        throw this.#neverError;
    }
    /**
     * @description Unimplemented
     */
    toU8a() {
        throw this.#neverError;
    }
}
