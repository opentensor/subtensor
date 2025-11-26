import { Tuple } from '../base/Tuple.js';
/**
 * @name Range
 * @description
 * Rust `Range<T>` representation
 */
export class Range extends Tuple {
    #rangeName;
    constructor(registry, Type, value, { rangeName = 'Range' } = {}) {
        super(registry, [Type, Type], value);
        this.#rangeName = rangeName;
    }
    static with(Type) {
        return class extends Range {
            constructor(registry, value) {
                super(registry, Type, value);
            }
        };
    }
    /**
     * @description Returns the starting range value
     */
    get start() {
        return this[0];
    }
    /**
     * @description Returns the ending range value
     */
    get end() {
        return this[1];
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return `${this.#rangeName}<${this.start.toRawType()}>`;
    }
}
