import type { ScryptParams } from './types.js';
interface Result {
    params: ScryptParams;
    password: Uint8Array;
    salt: Uint8Array;
}
export declare function scryptEncode(passphrase?: string | Uint8Array, salt?: Uint8Array, params?: ScryptParams, onlyJs?: boolean): Result;
export {};
