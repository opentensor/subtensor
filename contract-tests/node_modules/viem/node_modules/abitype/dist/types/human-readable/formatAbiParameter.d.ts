import type { AbiEventParameter, AbiParameter } from '../abi.js';
import type { IsNarrowable, Join } from '../types.js';
import type { AssertName } from './types/signatures.js';
/**
 * Formats {@link AbiParameter} to human-readable ABI parameter.
 *
 * @param abiParameter - ABI parameter
 * @returns Human-readable ABI parameter
 *
 * @example
 * type Result = FormatAbiParameter<{ type: 'address'; name: 'from'; }>
 * //   ^? type Result = 'address from'
 */
export type FormatAbiParameter<abiParameter extends AbiParameter | AbiEventParameter> = abiParameter extends {
    name?: infer name extends string;
    type: `tuple${infer array}`;
    components: infer components extends readonly AbiParameter[];
    indexed?: infer indexed extends boolean;
} ? FormatAbiParameter<{
    type: `(${Join<{
        [key in keyof components]: FormatAbiParameter<{
            type: components[key]['type'];
        } & (IsNarrowable<components[key]['name'], string> extends true ? {
            name: components[key]['name'];
        } : unknown) & (components[key] extends {
            components: readonly AbiParameter[];
        } ? {
            components: components[key]['components'];
        } : unknown)>;
    }, ', '>})${array}`;
} & (IsNarrowable<name, string> extends true ? {
    name: name;
} : unknown) & (IsNarrowable<indexed, boolean> extends true ? {
    indexed: indexed;
} : unknown)> : `${abiParameter['type']}${abiParameter extends {
    indexed: true;
} ? ' indexed' : ''}${abiParameter['name'] extends infer name extends string ? name extends '' ? '' : ` ${AssertName<name>}` : ''}`;
/**
 * Formats {@link AbiParameter} to human-readable ABI parameter.
 *
 * @param abiParameter - ABI parameter
 * @returns Human-readable ABI parameter
 *
 * @example
 * const result = formatAbiParameter({ type: 'address', name: 'from' })
 * //    ^? const result: 'address from'
 */
export declare function formatAbiParameter<const abiParameter extends AbiParameter | AbiEventParameter>(abiParameter: abiParameter): FormatAbiParameter<abiParameter>;
//# sourceMappingURL=formatAbiParameter.d.ts.map