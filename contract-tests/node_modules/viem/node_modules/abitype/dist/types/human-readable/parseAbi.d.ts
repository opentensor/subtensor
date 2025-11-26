import type { Abi } from '../abi.js';
import type { Error, Filter } from '../types.js';
import type { Signatures } from './types/signatures.js';
import type { ParseStructs } from './types/structs.js';
import type { ParseSignature } from './types/utils.js';
/**
 * Parses human-readable ABI into JSON {@link Abi}
 *
 * @param signatures - Human-readable ABI
 * @returns Parsed {@link Abi}
 *
 * @example
 * type Result = ParseAbi<
 *   // ^? type Result = readonly [{ name: "balanceOf"; type: "function"; stateMutability:...
 *   [
 *     'function balanceOf(address owner) view returns (uint256)',
 *     'event Transfer(address indexed from, address indexed to, uint256 amount)',
 *   ]
 * >
 */
export type ParseAbi<signatures extends readonly string[]> = string[] extends signatures ? Abi : signatures extends readonly string[] ? signatures extends Signatures<signatures> ? ParseStructs<signatures> extends infer sructs ? {
    [key in keyof signatures]: signatures[key] extends string ? ParseSignature<signatures[key], sructs> : never;
} extends infer mapped extends readonly unknown[] ? Filter<mapped, never> extends infer result ? result extends readonly [] ? never : result : never : never : never : never : never;
/**
 * Parses human-readable ABI into JSON {@link Abi}
 *
 * @param signatures - Human-Readable ABI
 * @returns Parsed {@link Abi}
 *
 * @example
 * const abi = parseAbi([
 *   //  ^? const abi: readonly [{ name: "balanceOf"; type: "function"; stateMutability:...
 *   'function balanceOf(address owner) view returns (uint256)',
 *   'event Transfer(address indexed from, address indexed to, uint256 amount)',
 * ])
 */
export declare function parseAbi<const signatures extends readonly string[]>(signatures: signatures['length'] extends 0 ? Error<'At least one signature required'> : Signatures<signatures> extends signatures ? signatures : Signatures<signatures>): ParseAbi<signatures>;
//# sourceMappingURL=parseAbi.d.ts.map