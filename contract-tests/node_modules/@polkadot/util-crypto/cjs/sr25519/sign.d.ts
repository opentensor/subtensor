import type { Keypair } from '../types.js';
/**
 * @name sr25519Sign
 * @description Returns message signature of `message`, using the supplied pair
 */
export declare function sr25519Sign(message: string | Uint8Array, { publicKey, secretKey }: Partial<Keypair>): Uint8Array;
