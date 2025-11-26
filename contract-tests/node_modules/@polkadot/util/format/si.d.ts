import type { SiDef } from '../types.js';
/** @internal */
export declare const SI_MID = 8;
/** @internal */
export declare const SI: SiDef[];
/** @internal */
export declare function findSi(type: string): SiDef;
/** @internal */
export declare function calcSi(text: string, decimals: number, forceUnit?: string): SiDef;
