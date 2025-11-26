import type { SignOptions } from '@polkadot/keyring/types';
import type { Registry } from '@polkadot/types-codec/types';
import type { IKeyringPair } from '../types/index.js';
export declare function sign(_registry: Registry, signerPair: IKeyringPair, u8a: Uint8Array, options?: SignOptions): Uint8Array;
export declare function signGeneral(registry: Registry, u8a: Uint8Array): Uint8Array;
