import { AbstractInt } from '../abstract/Int.js';
/**
 * @name UInt
 * @description
 * A generic unsigned integer codec. For Substrate all numbers are Little Endian encoded,
 * this handles the encoding and decoding of those numbers. Upon construction
 * the bitLength is provided and any additional use keeps the number to this
 * length. This extends `BN`, so all methods available on a normal `BN` object
 * is available here.
 * @noInheritDoc
 */
export class UInt extends AbstractInt {
    static with(bitLength, typeName) {
        return class extends UInt {
            constructor(registry, value) {
                super(registry, value, bitLength);
            }
            toRawType() {
                return typeName || super.toRawType();
            }
        };
    }
}
