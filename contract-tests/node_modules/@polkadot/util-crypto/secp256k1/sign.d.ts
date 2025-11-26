import type { Keypair } from '../types.js';
import type { HashType } from './types.js';
/**
 * @name secp256k1Sign
 * @description Returns message signature of `message`, using the supplied pair
 */
export declare function secp256k1Sign(message: Uint8Array | string, { secretKey }: Partial<Keypair>, hashType?: HashType, onlyJs?: boolean): Uint8Array;
