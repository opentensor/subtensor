import type { BN } from '../bn/bn.js';
import type { ToBn } from '../types.js';
interface Options {
    /**
     * @description The locale to use
     */
    locale?: string;
}
/**
 * @name formatNumber
 * @description Formats a number into string format with thousand separators
 */
export declare function formatNumber<ExtToBn extends ToBn>(value?: ExtToBn | BN | bigint | number | null, { locale }?: Options): string;
export {};
