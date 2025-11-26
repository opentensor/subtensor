import type { HashType } from './types.js';
/**
 * @name secp256k1Recover
 * @description Recovers a publicKey from the supplied signature
 */
export declare function secp256k1Recover(msgHash: string | Uint8Array, signature: string | Uint8Array, recovery: number, hashType?: HashType, onlyJs?: boolean): Uint8Array;
