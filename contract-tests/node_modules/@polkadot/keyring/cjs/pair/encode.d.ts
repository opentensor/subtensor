import type { PairInfo } from './types.js';
/**
 * Encode a pair with the latest generation format (generation 3)
 **/
export declare function encodePair({ publicKey, secretKey }: PairInfo, passphrase?: string): Uint8Array;
