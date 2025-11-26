"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.naclEncrypt = naclEncrypt;
const asU8a_js_1 = require("../random/asU8a.js");
const tweetnacl_js_1 = require("./tweetnacl.js");
/**
 * @name naclEncrypt
 * @summary Encrypts a message using the supplied secretKey and nonce
 * @description
 * Returns an encrypted message, using the `secretKey` and `nonce`. If the `nonce` was not supplied, a random value is generated.
 * @example
 * <BR>
 *
 * ```javascript
 * import { naclEncrypt } from '@polkadot/util-crypto';
 *
 * naclEncrypt([...], [...]); // => [...]
 * ```
 */
function naclEncrypt(message, secret, nonce = (0, asU8a_js_1.randomAsU8a)(24)) {
    return {
        encrypted: (0, tweetnacl_js_1.naclSecretbox)(message, nonce, secret),
        nonce
    };
}
