import type { StorageHasher } from '../../../interfaces/index.js';
export type HasherInput = string | Buffer | Uint8Array;
export type HasherFunction = (data: HasherInput) => Uint8Array;
/** @internal */
export declare function getHasher(hasher: StorageHasher): HasherFunction;
