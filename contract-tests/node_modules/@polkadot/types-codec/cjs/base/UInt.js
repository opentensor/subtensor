"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.UInt = void 0;
const Int_js_1 = require("../abstract/Int.js");
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
class UInt extends Int_js_1.AbstractInt {
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
exports.UInt = UInt;
