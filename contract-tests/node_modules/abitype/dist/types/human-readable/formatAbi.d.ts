import type { Abi } from '../abi.js';
import { type FormatAbiItem } from './formatAbiItem.js';
/**
 * Parses JSON ABI into human-readable ABI
 *
 * @param abi - ABI
 * @returns Human-readable ABI
 */
export type FormatAbi<abi extends Abi | readonly unknown[]> = Abi extends abi ? readonly string[] : abi extends readonly [] ? never : abi extends Abi ? {
    [key in keyof abi]: FormatAbiItem<abi[key]>;
} : readonly string[];
/**
 * Parses JSON ABI into human-readable ABI
 *
 * @param abi - ABI
 * @returns Human-readable ABI
 */
export declare function formatAbi<const abi extends Abi | readonly unknown[]>(abi: abi): FormatAbi<abi>;
//# sourceMappingURL=formatAbi.d.ts.map