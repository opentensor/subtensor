"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.base64Encode = exports.base64Decode = exports.isBase64 = exports.base64Validate = void 0;
const base_1 = require("@scure/base");
const helpers_js_1 = require("../base32/helpers.js");
const config = {
    chars: 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/',
    coder: base_1.base64,
    type: 'base64',
    withPadding: true
};
/**
 * @name base64Validate
 * @summary Validates a base64 value.
 * @description
 * Validates that the supplied value is valid base64
 */
exports.base64Validate = (0, helpers_js_1.createValidate)(config);
/**
 * @name isBase64
 * @description Checks if the input is in base64, returning true/false
 */
exports.isBase64 = (0, helpers_js_1.createIs)(exports.base64Validate);
/**
 * @name base64Decode
 * @summary Decodes a base64 value.
 * @description
 * From the provided input, decode the base64 and return the result as an `Uint8Array`.
 */
exports.base64Decode = (0, helpers_js_1.createDecode)(config, exports.base64Validate);
/**
 * @name base64Encode
 * @summary Creates a base64 value.
 * @description
 * From the provided input, create the base64 and return the result as a string.
 */
exports.base64Encode = (0, helpers_js_1.createEncode)(config);
