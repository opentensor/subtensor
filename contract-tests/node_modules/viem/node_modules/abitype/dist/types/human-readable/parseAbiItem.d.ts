import type { Abi } from '../abi.js';
import type { Narrow } from '../narrow.js';
import type { Error, Filter } from '../types.js';
import type { Signature, Signatures } from './types/signatures.js';
import type { ParseStructs } from './types/structs.js';
import type { ParseSignature } from './types/utils.js';
/**
 * Parses human-readable ABI item (e.g. error, event, function) into {@link Abi} item
 *
 * @param signature - Human-readable ABI item
 * @returns Parsed {@link Abi} item
 *
 * @example
 * type Result = ParseAbiItem<'function balanceOf(address owner) view returns (uint256)'>
 * //   ^? type Result = { name: "balanceOf"; type: "function"; stateMutability: "view";...
 *
 * @example
 * type Result = ParseAbiItem<
 *   // ^? type Result = { name: "foo"; type: "function"; stateMutability: "view"; inputs:...
 *   ['function foo(Baz bar) view returns (string)', 'struct Baz { string name; }']
 * >
 */
export type ParseAbiItem<signature extends string | readonly string[] | readonly unknown[]> = (signature extends string ? string extends signature ? Abi[number] : signature extends Signature<signature> ? ParseSignature<signature> : never : never) | (signature extends readonly string[] ? string[] extends signature ? Abi[number] : signature extends Signatures<signature> ? ParseStructs<signature> extends infer structs ? {
    [key in keyof signature]: ParseSignature<signature[key] extends string ? signature[key] : never, structs>;
} extends infer mapped extends readonly unknown[] ? Filter<mapped, never>[0] extends infer result ? result extends undefined ? never : result : never : never : never : never : never);
/**
 * Parses human-readable ABI item (e.g. error, event, function) into {@link Abi} item
 *
 * @param signature - Human-readable ABI item
 * @returns Parsed {@link Abi} item
 *
 * @example
 * const abiItem = parseAbiItem('function balanceOf(address owner) view returns (uint256)')
 * //    ^? const abiItem: { name: "balanceOf"; type: "function"; stateMutability: "view";...
 *
 * @example
 * const abiItem = parseAbiItem([
 *   //  ^? const abiItem: { name: "foo"; type: "function"; stateMutability: "view"; inputs:...
 *   'function foo(Baz bar) view returns (string)',
 *   'struct Baz { string name; }',
 * ])
 */
export declare function parseAbiItem<signature extends string | readonly string[] | readonly unknown[]>(signature: Narrow<signature> & ((signature extends string ? string extends signature ? unknown : Signature<signature> : never) | (signature extends readonly string[] ? signature extends readonly [] ? Error<'At least one signature required.'> : string[] extends signature ? unknown : Signatures<signature> : never))): ParseAbiItem<signature>;
//# sourceMappingURL=parseAbiItem.d.ts.map