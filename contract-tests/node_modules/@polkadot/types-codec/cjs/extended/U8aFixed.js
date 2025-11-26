"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.U8aFixed = void 0;
const util_1 = require("@polkadot/util");
const Raw_js_1 = require("../native/Raw.js");
/** @internal */
function decodeU8aFixed(value, bitLength) {
    const u8a = (0, util_1.u8aToU8a)(value);
    const byteLength = bitLength / 8;
    if (!u8a.length) {
        return [new Uint8Array(byteLength), 0];
    }
    if ((0, util_1.isU8a)(value) ? u8a.length < byteLength : u8a.length !== byteLength) {
        throw new Error(`Expected input with ${byteLength} bytes (${bitLength} bits), found ${u8a.length} bytes`);
    }
    return [u8a.subarray(0, byteLength), byteLength];
}
/**
 * @name U8aFixed
 * @description
 * A U8a that manages a a sequence of bytes up to the specified bitLength. Not meant
 * to be used directly, rather is should be subclassed with the specific lengths.
 */
class U8aFixed extends Raw_js_1.Raw {
    constructor(registry, value = new Uint8Array(), bitLength = 256) {
        const [u8a, decodedLength] = decodeU8aFixed(value, bitLength);
        super(registry, u8a, decodedLength);
    }
    static with(bitLength, typeName) {
        return class extends U8aFixed {
            constructor(registry, value) {
                super(registry, value, bitLength);
            }
            toRawType() {
                return typeName || super.toRawType();
            }
        };
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return `[u8;${this.length}]`;
    }
}
exports.U8aFixed = U8aFixed;
