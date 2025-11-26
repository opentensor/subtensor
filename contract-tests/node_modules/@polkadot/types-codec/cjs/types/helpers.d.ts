import type { BN } from '@polkadot/util';
import type { HexString } from '@polkadot/util/types';
import type { Codec } from './codec.js';
export type AnyJson = string | number | boolean | null | undefined | AnyJson[] | {
    [index: string]: AnyJson;
};
export type AnyFunction = (...args: any[]) => any;
export type AnyNumber = BN | bigint | Uint8Array | number | string;
export type AnyFloat = Number | number | Uint8Array | string;
export type AnyString = String | string;
export type AnyBool = Boolean | boolean;
export type AnyTuple = Codec[];
export type AnyU8a = Uint8Array | number[] | string;
export type UIntBitLength = 8 | 16 | 32 | 64 | 128 | 256;
export type U8aBitLength = 8 | 16 | 32 | 64 | 128 | 160 | 256 | 264 | 512 | 520 | 1024 | 2048;
export type AnyTupleValue = Exclude<AnyU8a, string> | HexString | (Codec | AnyU8a | AnyNumber | AnyString | undefined | null)[];
export interface ToString {
    toString: () => string;
}
export interface ToBn {
    toBn: () => BN;
}
export interface DefinitionSetter<T> {
    definition?: T | undefined;
    setDefinition?: (d: T) => T;
}
export type LookupString = `Lookup${number}`;
