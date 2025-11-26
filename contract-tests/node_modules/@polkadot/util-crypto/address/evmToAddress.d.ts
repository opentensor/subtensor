import type { HashType } from '../secp256k1/types.js';
import type { Prefix } from './types.js';
/**
 * @name evmToAddress
 * @summary Converts an EVM address to its corresponding SS58 address.
 */
export declare function evmToAddress(evmAddress: string | Uint8Array, ss58Format?: Prefix, hashType?: HashType): string;
