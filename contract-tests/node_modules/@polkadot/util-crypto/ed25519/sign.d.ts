import type { Keypair } from '../types.js';
/**
 * @name ed25519Sign
 * @summary Signs a message using the supplied secretKey
 * @description
 * Returns message signature of `message`, using the `secretKey`.
 * @example
 * <BR>
 *
 * ```javascript
 * import { ed25519Sign } from '@polkadot/util-crypto';
 *
 * ed25519Sign([...], [...]); // => [...]
 * ```
 */
export declare function ed25519Sign(message: string | Uint8Array, { publicKey, secretKey }: Partial<Keypair>, onlyJs?: boolean): Uint8Array;
