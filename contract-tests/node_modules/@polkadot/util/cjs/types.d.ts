import type { BN } from './bn/bn.js';
/** An interface that defines an actual JS class */
export interface Class<T = any, A extends unknown[] = any[]> {
    prototype: T;
    new (...args: A): T;
    hasOwnProperty(prop: string): boolean;
    isPrototypeOf(other: unknown): boolean;
}
export type Constructor<T = any, A extends unknown[] = any[]> = Class<T, A>;
export interface ToBigInt {
    toBigInt: () => bigint;
}
export interface ToBn {
    toBn: () => BN;
}
export interface SiDef {
    power: number;
    text: string;
    value: string;
}
export interface Logger {
    debug: (...values: unknown[]) => void;
    error: (...values: unknown[]) => void;
    log: (...values: unknown[]) => void;
    noop: (...values: unknown[]) => void;
    warn: (...values: unknown[]) => void;
}
export interface ToBnOptions {
    /** Convert in LE format */
    isLe?: boolean;
    /** Number is signed, apply two's complement */
    isNegative?: boolean;
}
export interface NumberOptions extends ToBnOptions {
    /** Limit to the specified bitLength, despite input length */
    bitLength?: number;
}
export interface Time {
    days: number;
    hours: number;
    minutes: number;
    seconds: number;
    milliseconds: number;
}
export type Memoized<F> = F & {
    unmemoize: (...args: unknown[]) => void;
};
export type AnyString = string | String;
export type HexDigit = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | 'a' | 'b' | 'c' | 'd' | 'e' | 'f';
export type HexString = `0x${string}`;
export interface BufferObject extends Uint8Array {
    equals: (otherBuffer: Uint8Array) => boolean;
    readDoubleLE: (offset?: number) => number;
}
export interface BufferClass extends Class<BufferObject> {
    from: <T = BufferObject>(value: unknown) => T;
    isBuffer: (value: unknown) => boolean;
}
export type U8aLike = number[] | Uint8Array | AnyString;
export interface Observable {
    next: (...params: unknown[]) => unknown;
}
