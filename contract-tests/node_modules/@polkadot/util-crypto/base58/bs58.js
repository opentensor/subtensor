import { base58 } from '@scure/base';
import { createDecode, createEncode, createIs, createValidate } from '../base32/helpers.js';
const config = {
    chars: '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz',
    coder: base58,
    ipfs: 'z',
    type: 'base58'
};
/**
 * @name base58Validate
 * @summary Validates a base58 value.
 * @description
 * Validates that the supplied value is valid base58, throwing exceptions if not
 */
export const base58Validate = /*#__PURE__*/ createValidate(config);
/**
 * @name base58Decode
 * @summary Decodes a base58 value.
 * @description
 * From the provided input, decode the base58 and return the result as an `Uint8Array`.
 */
export const base58Decode = /*#__PURE__*/ createDecode(config, base58Validate);
/**
* @name base58Encode
* @summary Creates a base58 value.
* @description
* From the provided input, create the base58 and return the result as a string.
*/
export const base58Encode = /*#__PURE__*/ createEncode(config);
/**
* @name isBase58
* @description Checks if the input is in base58, returning true/false
*/
export const isBase58 = /*#__PURE__*/ createIs(base58Validate);
