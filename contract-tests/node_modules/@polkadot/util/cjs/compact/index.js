"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.compactToU8a = exports.compactStripLength = exports.compactFromU8aLim = exports.compactFromU8a = exports.compactAddLength = void 0;
/**
 * @description
 * Encoding and decoding of parity-codec compact numbers. The codec is created
 * to take up the least amount of space for a specific number. It performs the
 * same function as Length, however differs in that it uses a variable number of
 * bytes to do the actual encoding. From the Rust implementation for compact
 * encoding:
 *
 *     0b00 00 00 00 / 00 00 00 00 / 00 00 00 00 / 00 00 00 00
 * (0 ... 2**6 - 1)    (u8)
 *     xx xx xx 00
 * (2**6 ... 2**14 - 1)  (u8, u16)  low LH high
 *     yL yL yL 01 / yH yH yH yL
 * (2**14 ... 2**30 - 1)  (u16, u32)  low LMMH high
 *     zL zL zL 10 / zM zM zM zL / zM zM zM zM / zH zH zH zM
 * (2**30 ... 2**536 - 1)  (u32, u64, u128, U256, U512, U520) straight LE-encoded
 *     nn nn nn 11 [ / zz zz zz zz ]{4 + n}
 *
 * Note: we use *LOW BITS* of the LSB in LE encoding to encode the 2 bit key.
 */
var addLength_js_1 = require("./addLength.js");
Object.defineProperty(exports, "compactAddLength", { enumerable: true, get: function () { return addLength_js_1.compactAddLength; } });
var fromU8a_js_1 = require("./fromU8a.js");
Object.defineProperty(exports, "compactFromU8a", { enumerable: true, get: function () { return fromU8a_js_1.compactFromU8a; } });
Object.defineProperty(exports, "compactFromU8aLim", { enumerable: true, get: function () { return fromU8a_js_1.compactFromU8aLim; } });
var stripLength_js_1 = require("./stripLength.js");
Object.defineProperty(exports, "compactStripLength", { enumerable: true, get: function () { return stripLength_js_1.compactStripLength; } });
var toU8a_js_1 = require("./toU8a.js");
Object.defineProperty(exports, "compactToU8a", { enumerable: true, get: function () { return toU8a_js_1.compactToU8a; } });
