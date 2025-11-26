import type { KeyringPair, KeyringPairs } from './types.js';
export declare class Pairs implements KeyringPairs {
    #private;
    add(pair: KeyringPair): KeyringPair;
    all(): KeyringPair[];
    get(address: string | Uint8Array): KeyringPair;
    remove(address: string | Uint8Array): void;
}
