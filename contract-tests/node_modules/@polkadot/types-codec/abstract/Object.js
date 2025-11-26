/**
 * @name Object
 * @description A type extends the Base class, when it holds a value
 */
export class AbstractObject {
    registry;
    createdAtHash;
    initialU8aLength;
    isStorageFallback;
    $;
    constructor(registry, value, initialU8aLength) {
        this.$ = value;
        this.initialU8aLength = initialU8aLength;
        this.registry = registry;
    }
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength() {
        return this.toU8a().length;
    }
    /**
     * @description returns a hash of the contents
     */
    get hash() {
        return this.registry.hash(this.toU8a());
    }
    /**
     * @description Returns the string representation of the value
     */
    toString() {
        return this.$.toString();
    }
    /**
     * @description Return the internal value (JS-aligned, same result as $)
     */
    valueOf() {
        return this.$;
    }
}
