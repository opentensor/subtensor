import type { Prefix } from './types.js';
/**
 * @name deriveAddress
 * @summary Creates a sr25519 derived address from the supplied and path.
 * @description
 * Creates a sr25519 derived address based on the input address/publicKey and the uri supplied.
 */
export declare function deriveAddress(who: string | Uint8Array, suri: string, ss58Format?: Prefix): string;
