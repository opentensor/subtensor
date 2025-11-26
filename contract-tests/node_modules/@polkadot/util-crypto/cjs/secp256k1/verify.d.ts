import type { HashType } from './types.js';
/**
 * @name secp256k1Verify
 * @description Verifies the signature of `message`, using the supplied pair
 */
export declare function secp256k1Verify(msgHash: string | Uint8Array, signature: string | Uint8Array, address: string | Uint8Array, hashType?: HashType, onlyJs?: boolean): boolean;
