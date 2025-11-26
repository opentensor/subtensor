import type { HexString } from '../types.js';
interface Registry {
    get: (...params: never[]) => any;
}
interface Codec {
    readonly registry: Registry;
    toHex(...params: never[]): HexString;
    toHuman(...params: never[]): unknown;
    toU8a: (...params: never[]) => Uint8Array;
}
export declare function isCodec<T extends Codec = Codec>(value?: unknown): value is T;
export {};
