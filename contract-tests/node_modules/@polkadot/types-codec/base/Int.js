import { AbstractInt } from '../abstract/Int.js';
/**
 * @name Int
 * @description
 * A generic signed integer codec. For Substrate all numbers are Little Endian encoded,
 * this handles the encoding and decoding of those numbers. Upon construction
 * the bitLength is provided and any additional use keeps the number to this
 * length. This extends `BN`, so all methods available on a normal `BN` object
 * is available here.
 * @noInheritDoc
 */
export class Int extends AbstractInt {
    constructor(registry, value = 0, bitLength) {
        super(registry, value, bitLength, true);
    }
    static with(bitLength, typeName) {
        return class extends Int {
            constructor(registry, value) {
                super(registry, value, bitLength);
            }
            toRawType() {
                return typeName || super.toRawType();
            }
        };
    }
}
