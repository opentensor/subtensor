import type { AbiParameter } from '../abi.js';
import type { Narrow } from '../narrow.js';
import type { Error, Filter } from '../types.js';
import type { IsStructSignature, Modifier } from './types/signatures.js';
import type { ParseStructs } from './types/structs.js';
import type { ParseAbiParameter as ParseAbiParameter_ } from './types/utils.js';
/**
 * Parses human-readable ABI parameter into {@link AbiParameter}
 *
 * @param param - Human-readable ABI parameter
 * @returns Parsed {@link AbiParameter}
 *
 * @example
 * type Result = ParseAbiParameter<'address from'>
 * //   ^? type Result = { type: "address"; name: "from"; }
 *
 * @example
 * type Result = ParseAbiParameter<
 *   // ^? type Result = { type: "tuple"; components: [{ type: "string"; name:...
 *   ['Baz bar', 'struct Baz { string name; }']
 * >
 */
export type ParseAbiParameter<param extends string | readonly string[] | readonly unknown[]> = (param extends string ? param extends '' ? never : string extends param ? AbiParameter : ParseAbiParameter_<param, {
    modifier: Modifier;
}> : never) | (param extends readonly string[] ? string[] extends param ? AbiParameter : ParseStructs<param> extends infer structs ? {
    [key in keyof param]: param[key] extends string ? IsStructSignature<param[key]> extends true ? never : ParseAbiParameter_<param[key], {
        modifier: Modifier;
        structs: structs;
    }> : never;
} extends infer mapped extends readonly unknown[] ? Filter<mapped, never>[0] extends infer result ? result extends undefined ? never : result : never : never : never : never);
/**
 * Parses human-readable ABI parameter into {@link AbiParameter}
 *
 * @param param - Human-readable ABI parameter
 * @returns Parsed {@link AbiParameter}
 *
 * @example
 * const abiParameter = parseAbiParameter('address from')
 * //    ^? const abiParameter: { type: "address"; name: "from"; }
 *
 * @example
 * const abiParameter = parseAbiParameter([
 *   //  ^? const abiParameter: { type: "tuple"; components: [{ type: "string"; name:...
 *   'Baz bar',
 *   'struct Baz { string name; }',
 * ])
 */
export declare function parseAbiParameter<param extends string | readonly string[] | readonly unknown[]>(param: Narrow<param> & ((param extends string ? param extends '' ? Error<'Empty string is not allowed.'> : unknown : never) | (param extends readonly string[] ? param extends readonly [] ? Error<'At least one parameter required.'> : string[] extends param ? unknown : unknown : never))): ParseAbiParameter<param>;
//# sourceMappingURL=parseAbiParameter.d.ts.map