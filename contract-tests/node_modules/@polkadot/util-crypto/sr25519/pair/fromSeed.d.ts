import type { Keypair } from '../../types.js';
/**
 * @name sr25519PairFromSeed
 * @description Returns a object containing a `publicKey` & `secretKey` generated from the supplied seed.
 */
export declare function sr25519PairFromSeed(seed: string | Uint8Array): Keypair;
