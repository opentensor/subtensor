import type { U8aLike } from '@polkadot/util/types';
export type { U8aLike } from '@polkadot/util/types';
interface Coder {
    decode: (value: string) => Uint8Array;
    encode: (value: Uint8Array) => string;
}
interface Config {
    chars: string;
    coder: Coder;
    ipfs?: string;
    regex?: RegExp;
    type: string;
    withPadding?: boolean;
}
type DecodeFn = (value: string, ipfsCompat?: boolean) => Uint8Array;
type EncodeFn = (value: U8aLike, ipfsCompat?: boolean) => string;
type ValidateFn = (value?: unknown, ipfsCompat?: boolean) => value is string;
/** @internal */
export declare function createDecode({ coder, ipfs }: Config, validate: ValidateFn): DecodeFn;
/** @internal */
export declare function createEncode({ coder, ipfs }: Config): EncodeFn;
/** @internal */
export declare function createIs(validate: ValidateFn): ValidateFn;
/** @internal */
export declare function createValidate({ chars, ipfs, type, withPadding }: Config): ValidateFn;
