"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.naclDecrypt = naclDecrypt;
const tweetnacl_js_1 = require("./tweetnacl.js");
/**
 * @name naclDecrypt
 * @summary Decrypts a message using the supplied secretKey and nonce
 * @description
 * Returns an decrypted message, using the `secret` and `nonce`.
 * @example
 * <BR>
 *
 * ```javascript
 * import { naclDecrypt } from '@polkadot/util-crypto';
 *
 * naclDecrypt([...], [...], [...]); // => [...]
 * ```
 */
function naclDecrypt(encrypted, nonce, secret) {
    return (0, tweetnacl_js_1.naclSecretboxOpen)(encrypted, nonce, secret);
}
