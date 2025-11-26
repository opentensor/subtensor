import type { ScryptParams } from './types.js';
export declare function scryptToU8a(salt: Uint8Array, { N, p, r }: ScryptParams): Uint8Array;
