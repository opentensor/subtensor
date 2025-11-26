import type { ScryptParams } from './types.js';
interface Result {
    params: ScryptParams;
    salt: Uint8Array;
}
export declare function scryptFromU8a(data: Uint8Array): Result;
export {};
