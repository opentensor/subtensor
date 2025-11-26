"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isBase58 = exports.base58Encode = exports.base58Decode = exports.base58Validate = void 0;
const base_1 = require("@scure/base");
const helpers_js_1 = require("../base32/helpers.js");
const config = {
    chars: '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz',
    coder: base_1.base58,
    ipfs: 'z',
    type: 'base58'
};
/**
 * @name base58Validate
 * @summary Validates a base58 value.
 * @description
 * Validates that the supplied value is valid base58, throwing exceptions if not
 */
exports.base58Validate = (0, helpers_js_1.createValidate)(config);
/**
 * @name base58Decode
 * @summary Decodes a base58 value.
 * @description
 * From the provided input, decode the base58 and return the result as an `Uint8Array`.
 */
exports.base58Decode = (0, helpers_js_1.createDecode)(config, exports.base58Validate);
/**
* @name base58Encode
* @summary Creates a base58 value.
* @description
* From the provided input, create the base58 and return the result as a string.
*/
exports.base58Encode = (0, helpers_js_1.createEncode)(config);
/**
* @name isBase58
* @description Checks if the input is in base58, returning true/false
*/
exports.isBase58 = (0, helpers_js_1.createIs)(exports.base58Validate);
