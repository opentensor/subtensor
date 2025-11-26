import type { Prefix } from './types.js';
export declare function decodeAddress(encoded?: string | Uint8Array | null, ignoreChecksum?: boolean, ss58Format?: Prefix): Uint8Array;
