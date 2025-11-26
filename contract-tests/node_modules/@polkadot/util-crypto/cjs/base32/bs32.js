"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.base32Encode = exports.base32Decode = exports.isBase32 = exports.base32Validate = void 0;
const base_1 = require("@scure/base");
const helpers_js_1 = require("./helpers.js");
const chars = 'abcdefghijklmnopqrstuvwxyz234567';
const config = {
    chars,
    coder: base_1.utils.chain(
    // We define our own chain, the default base32 has padding
    base_1.utils.radix2(5), base_1.utils.alphabet(chars), {
        decode: (input) => input.split(''),
        encode: (input) => input.join('')
    }),
    ipfs: 'b',
    type: 'base32'
};
/**
 * @name base32Validate
 * @summary Validates a base32 value.
 * @description
 * Validates that the supplied value is valid base32, throwing exceptions if not
 */
exports.base32Validate = (0, helpers_js_1.createValidate)(config);
/**
* @name isBase32
* @description Checks if the input is in base32, returning true/false
*/
exports.isBase32 = (0, helpers_js_1.createIs)(exports.base32Validate);
/**
 * @name base32Decode
 * @summary Delookup a base32 value.
 * @description
 * From the provided input, decode the base32 and return the result as an `Uint8Array`.
 */
exports.base32Decode = (0, helpers_js_1.createDecode)(config, exports.base32Validate);
/**
* @name base32Encode
* @summary Creates a base32 value.
* @description
* From the provided input, create the base32 and return the result as a string.
*/
exports.base32Encode = (0, helpers_js_1.createEncode)(config);
