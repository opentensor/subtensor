import type { BN } from '../bn/bn.js';
import type { SiDef, ToBn } from '../types.js';
interface Defaults {
    decimals: number;
    unit: string;
}
interface SetDefaults {
    decimals?: number[] | number;
    unit?: string[] | string;
}
interface Options {
    /**
     * @description The number of decimals
     */
    decimals?: number;
    /**
     * @description Format the number with this specific unit
     */
    forceUnit?: string;
    /**
     * @description Returns value using all available decimals
     */
    withAll?: boolean;
    /**
     * @description Format with SI, i.e. m/M/etc. (default = true)
     */
    withSi?: boolean;
    /**
     * @description Format with full SI, i.e. mili/Mega/etc.
     */
    withSiFull?: boolean;
    /**
     * @description Add the unit (useful in Balance formats)
     */
    withUnit?: boolean | string;
    /**
     * @description Returns all trailing zeros, otherwise removes (default = true)
     */
    withZero?: boolean;
    /**
     * @description The locale to use
     */
    locale?: string;
}
interface BalanceFormatter {
    <ExtToBn extends ToBn>(input?: number | string | BN | bigint | ExtToBn, options?: Options): string;
    calcSi(text: string, decimals?: number): SiDef;
    findSi(type: string): SiDef;
    getDefaults(): Defaults;
    getOptions(decimals?: number): SiDef[];
    setDefaults(defaults: SetDefaults): void;
}
export declare const formatBalance: BalanceFormatter;
export {};
