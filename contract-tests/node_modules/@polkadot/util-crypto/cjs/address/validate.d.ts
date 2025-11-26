import type { Prefix } from './types.js';
export declare function validateAddress(encoded?: string | null, ignoreChecksum?: boolean, ss58Format?: Prefix): encoded is string;
