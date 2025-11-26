import type { BN } from '../bn/bn.js';
import type { ToBn } from '../types.js';
/**
 * @name formatElapsed
 * @description Formats an elapsed value into s, m, h or day segments
 */
export declare function formatElapsed<ExtToBn extends ToBn>(now?: Date | null, value?: bigint | BN | ExtToBn | Date | number | null): string;
