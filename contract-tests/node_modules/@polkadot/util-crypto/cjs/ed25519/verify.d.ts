/**
 * @name ed25519Sign
 * @summary Verifies the signature on the supplied message.
 * @description
 * Verifies the `signature` on `message` with the supplied `publicKey`. Returns `true` on sucess, `false` otherwise.
 * @example
 * <BR>
 *
 * ```javascript
 * import { ed25519Verify } from '@polkadot/util-crypto';
 *
 * ed25519Verify([...], [...], [...]); // => true/false
 * ```
 */
export declare function ed25519Verify(message: string | Uint8Array, signature: string | Uint8Array, publicKey: string | Uint8Array, onlyJs?: boolean): boolean;
