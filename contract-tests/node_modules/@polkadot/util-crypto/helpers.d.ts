import type { HexString } from '@polkadot/util/types';
export type { HexString } from '@polkadot/util/types';
interface DualHash {
    256: (u8a: Uint8Array) => Uint8Array;
    512: (u8a: Uint8Array) => Uint8Array;
}
/** @internal */
export declare function createAsHex<T extends (...args: never[]) => Uint8Array>(fn: T): (...args: Parameters<T>) => HexString;
/** @internal */
export declare function createBitHasher(bitLength: 256 | 512, fn: (data: string | Uint8Array, bitLength: 256 | 512, onlyJs?: boolean) => Uint8Array): (data: string | Uint8Array, onlyJs?: boolean) => Uint8Array;
/** @internal */
export declare function createDualHasher(wa: DualHash, js: DualHash): (value: string | Uint8Array, bitLength?: 256 | 512, onlyJs?: boolean) => Uint8Array;
