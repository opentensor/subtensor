import type { Prefix } from './types.js';
export declare function isAddress(address?: string | null, ignoreChecksum?: boolean, ss58Format?: Prefix): address is string;
