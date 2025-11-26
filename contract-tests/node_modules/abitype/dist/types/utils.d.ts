import type { Abi, AbiParameter, AbiParameterKind, AbiStateMutability, AbiType, MBits, SolidityArray, SolidityBytes, SolidityFixedArrayRange, SolidityFixedArraySizeLookup, SolidityInt, SolidityTuple, TypedData, TypedDataParameter, TypedDataType } from './abi.js';
import type { ResolvedRegister } from './register.js';
import type { Error, Merge, Pretty, Tuple } from './types.js';
/**
 * Converts {@link AbiType} to corresponding TypeScript primitive type.
 *
 * Does not include full array or tuple conversion. Use {@link AbiParameterToPrimitiveType} to fully convert arrays and tuples.
 *
 * @param abiType - {@link AbiType} to convert to TypeScript representation
 * @param abiParameterKind - Optional {@link AbiParameterKind} to narrow by parameter type
 * @returns TypeScript primitive type
 */
export type AbiTypeToPrimitiveType<abiType extends AbiType, abiParameterKind extends AbiParameterKind = AbiParameterKind> = abiType extends SolidityBytes ? PrimitiveTypeLookup[abiType][abiParameterKind] : PrimitiveTypeLookup[abiType];
interface PrimitiveTypeLookup extends SolidityIntMap, SolidityByteMap, SolidityArrayMap {
    address: ResolvedRegister['addressType'];
    bool: boolean;
    function: `${ResolvedRegister['addressType']}${string}`;
    string: string;
    tuple: Record<string, unknown>;
}
type SolidityIntMap = {
    [_ in SolidityInt]: _ extends `${'u' | ''}int${infer bits extends keyof BitsTypeLookup}` ? BitsTypeLookup[bits] : never;
};
type SolidityByteMap = {
    [_ in SolidityBytes]: ResolvedRegister['bytesType'];
};
type SolidityArrayMap = {
    [_ in SolidityArray]: readonly unknown[];
};
type GreaterThan48Bits = Exclude<MBits, 8 | 16 | 24 | 32 | 40 | 48 | NoBits>;
type LessThanOrEqualTo48Bits = Exclude<MBits, GreaterThan48Bits | NoBits>;
type NoBits = '';
type BitsTypeLookup = {
    [key in MBits]: ResolvedRegister[key extends LessThanOrEqualTo48Bits ? 'intType' : 'bigIntType'];
};
/**
 * Converts {@link AbiParameter} to corresponding TypeScript primitive type.
 *
 * @param abiParameter - {@link AbiParameter} to convert to TypeScript representation
 * @param abiParameterKind - Optional {@link AbiParameterKind} to narrow by parameter type
 * @returns TypeScript primitive type
 */
export type AbiParameterToPrimitiveType<abiParameter extends AbiParameter | {
    name: string;
    type: unknown;
}, abiParameterKind extends AbiParameterKind = AbiParameterKind> = abiParameter['type'] extends AbiBasicType ? AbiTypeToPrimitiveType<abiParameter['type'], abiParameterKind> : abiParameter extends {
    type: SolidityTuple;
    components: infer components extends readonly AbiParameter[];
} ? AbiComponentsToPrimitiveType<components, abiParameterKind> : MaybeExtractArrayParameterType<abiParameter['type']> extends [
    infer head extends string,
    infer size
] ? AbiArrayToPrimitiveType<abiParameter, abiParameterKind, head, size> : ResolvedRegister['strictAbiType'] extends true ? Error<`Unknown type '${abiParameter['type'] & string}'.`> : abiParameter extends {
    components: Error<string>;
} ? abiParameter['components'] : unknown;
type AbiBasicType = Exclude<AbiType, SolidityTuple | SolidityArray>;
type AbiComponentsToPrimitiveType<components extends readonly AbiParameter[], abiParameterKind extends AbiParameterKind> = components extends readonly [] ? [] : components[number]['name'] extends Exclude<components[number]['name'] & string, undefined | ''> ? {
    [component in components[number] as component['name'] & {}]: AbiParameterToPrimitiveType<component, abiParameterKind>;
} : {
    [key in keyof components]: AbiParameterToPrimitiveType<components[key], abiParameterKind>;
};
type MaybeExtractArrayParameterType<type> = 
/**
 * First, infer `Head` against a known size type (either fixed-length array value or `""`).
 *
 * | Input           | Head         |
 * | --------------- | ------------ |
 * | `string[]`      | `string`     |
 * | `string[][][3]` | `string[][]` |
 */
type extends `${infer head}[${'' | `${SolidityFixedArrayRange}`}]` ? type extends `${head}[${infer size}]` ? [head, size] : undefined : undefined;
type AbiArrayToPrimitiveType<abiParameter extends AbiParameter | {
    name: string;
    type: unknown;
}, abiParameterKind extends AbiParameterKind, head extends string, size> = size extends keyof SolidityFixedArraySizeLookup ? Tuple<AbiParameterToPrimitiveType<Merge<abiParameter, {
    type: head;
}>, abiParameterKind>, SolidityFixedArraySizeLookup[size]> : readonly AbiParameterToPrimitiveType<Merge<abiParameter, {
    type: head;
}>, abiParameterKind>[];
/**
 * Converts array of {@link AbiParameter} to corresponding TypeScript primitive types.
 *
 * @param abiParameters - Array of {@link AbiParameter} to convert to TypeScript representations
 * @param abiParameterKind - Optional {@link AbiParameterKind} to narrow by parameter type
 * @returns Array of TypeScript primitive types
 */
export type AbiParametersToPrimitiveTypes<abiParameters extends readonly AbiParameter[], abiParameterKind extends AbiParameterKind = AbiParameterKind> = Pretty<{
    [key in keyof abiParameters]: AbiParameterToPrimitiveType<abiParameters[key], abiParameterKind>;
}>;
/**
 * Checks if type is {@link Abi}.
 *
 * @param abi - {@link Abi} to check
 * @returns Boolean for whether {@link abi} is {@link Abi}
 */
export type IsAbi<abi> = abi extends Abi ? true : false;
/**
 * Extracts all {@link AbiFunction} types from {@link Abi}.
 *
 * @param abi - {@link Abi} to extract functions from
 * @param abiStateMutability - {@link AbiStateMutability} to filter by
 * @returns All {@link AbiFunction} types from {@link Abi}
 */
export type ExtractAbiFunctions<abi extends Abi, abiStateMutability extends AbiStateMutability = AbiStateMutability> = Extract<abi[number], {
    type: 'function';
    stateMutability: abiStateMutability;
}>;
/**
 * Extracts all {@link AbiFunction} names from {@link Abi}.
 *
 * @param abi - {@link Abi} to extract function names from
 * @param abiStateMutability - {@link AbiStateMutability} to filter by
 * @returns Union of function names
 */
export type ExtractAbiFunctionNames<abi extends Abi, abiStateMutability extends AbiStateMutability = AbiStateMutability> = ExtractAbiFunctions<abi, abiStateMutability>['name'];
/**
 * Extracts {@link AbiFunction} with name from {@link Abi}.
 *
 * @param abi - {@link Abi} to extract {@link AbiFunction} from
 * @param functionName - String name of function to extract from {@link Abi}
 * @param abiStateMutability - {@link AbiStateMutability} to filter by
 * @returns Matching {@link AbiFunction}
 */
export type ExtractAbiFunction<abi extends Abi, functionName extends ExtractAbiFunctionNames<abi>, abiStateMutability extends AbiStateMutability = AbiStateMutability> = Extract<ExtractAbiFunctions<abi, abiStateMutability>, {
    name: functionName;
}>;
/**
 * Extracts all {@link AbiEvent} types from {@link Abi}.
 *
 * @param abi - {@link Abi} to extract events from
 * @returns All {@link AbiEvent} types from {@link Abi}
 */
export type ExtractAbiEvents<abi extends Abi> = Extract<abi[number], {
    type: 'event';
}>;
/**
 * Extracts all {@link AbiEvent} names from {@link Abi}.
 *
 * @param abi - {@link Abi} to extract event names from
 * @returns Union of event names
 */
export type ExtractAbiEventNames<abi extends Abi> = ExtractAbiEvents<abi>['name'];
/**
 * Extracts {@link AbiEvent} with name from {@link Abi}.
 *
 * @param abi - {@link Abi} to extract {@link AbiEvent} from
 * @param eventName - String name of event to extract from {@link Abi}
 * @returns Matching {@link AbiEvent}
 */
export type ExtractAbiEvent<abi extends Abi, eventName extends ExtractAbiEventNames<abi>> = Extract<ExtractAbiEvents<abi>, {
    name: eventName;
}>;
/**
 * Extracts all {@link AbiError} types from {@link Abi}.
 *
 * @param abi - {@link Abi} to extract errors from
 * @returns All {@link AbiError} types from {@link Abi}
 */
export type ExtractAbiErrors<abi extends Abi> = Extract<abi[number], {
    type: 'error';
}>;
/**
 * Extracts all {@link AbiError} names from {@link Abi}.
 *
 * @param abi - {@link Abi} to extract error names from
 * @returns Union of error names
 */
export type ExtractAbiErrorNames<abi extends Abi> = ExtractAbiErrors<abi>['name'];
/**
 * Extracts {@link AbiError} with name from {@link Abi}.
 *
 * @param abi - {@link Abi} to extract {@link AbiError} from
 * @param errorName - String name of error to extract from {@link Abi}
 * @returns Matching {@link AbiError}
 */
export type ExtractAbiError<abi extends Abi, errorName extends ExtractAbiErrorNames<abi>> = Extract<ExtractAbiErrors<abi>, {
    name: errorName;
}>;
/**
 * Checks if type is {@link TypedData}.
 *
 * @param typedData - {@link TypedData} to check
 * @returns Boolean for whether {@link typedData} is {@link TypedData}
 */
export type IsTypedData<typedData> = typedData extends TypedData ? {
    [key in keyof typedData]: {
        [key2 in typedData[key][number] as key2['type'] extends keyof typedData ? never : key2['type'] extends `${keyof typedData & string}[${string}]` ? never : key2['type'] extends TypedDataType ? never : key2['name']]: false;
    };
} extends {
    [key in keyof typedData]: Record<string, never>;
} ? true : false : false;
/**
 * Converts {@link typedData} to corresponding TypeScript primitive types.
 *
 * @param typedData - {@link TypedData} to convert
 * @param abiParameterKind - Optional {@link AbiParameterKind} to narrow by parameter type
 * @returns Union of TypeScript primitive types
 */
export type TypedDataToPrimitiveTypes<typedData extends TypedData, abiParameterKind extends AbiParameterKind = AbiParameterKind, keyReferences extends {
    [_: string]: unknown;
} | unknown = unknown> = {
    [key in keyof typedData]: {
        [key2 in typedData[key][number] as key2['name']]: key2['type'] extends key ? Error<`Cannot convert self-referencing struct '${key2['type']}' to primitive type.`> : key2['type'] extends keyof typedData ? key2['type'] extends keyof keyReferences ? Error<`Circular reference detected. '${key2['type']}' is a circular reference.`> : TypedDataToPrimitiveTypes<Exclude<typedData, key>, abiParameterKind, keyReferences & {
            [_ in key2['type'] | key]: true;
        }>[key2['type']] : key2['type'] extends `${infer type extends keyof typedData & string}[${infer tail}]` ? AbiParameterToPrimitiveType<{
            name: key2['name'];
            type: `tuple[${tail}]`;
            components: _TypedDataParametersToAbiParameters<typedData[type], typedData, keyReferences & {
                [_ in type | key]: true;
            }>;
        }, abiParameterKind> : key2['type'] extends TypedDataType ? AbiParameterToPrimitiveType<key2, abiParameterKind> : Error<`Cannot convert unknown type '${key2['type']}' to primitive type.`>;
    };
} & unknown;
type _TypedDataParametersToAbiParameters<typedDataParameters extends readonly TypedDataParameter[], typedData extends TypedData, keyReferences extends {
    [_: string]: unknown;
} | unknown = unknown> = {
    [key in keyof typedDataParameters]: typedDataParameters[key] extends infer typedDataParameter extends {
        name: string;
        type: unknown;
    } ? typedDataParameter['type'] extends keyof typedData & string ? {
        name: typedDataParameter['name'];
        type: 'tuple';
        components: typedDataParameter['type'] extends keyof keyReferences ? Error<`Circular reference detected. '${typedDataParameter['type']}' is a circular reference.`> : _TypedDataParametersToAbiParameters<typedData[typedDataParameter['type']], typedData, keyReferences & {
            [_ in typedDataParameter['type']]: true;
        }>;
    } : typedDataParameter['type'] extends `${infer type extends keyof typedData & string}[${infer tail}]` ? {
        name: typedDataParameter['name'];
        type: `tuple[${tail}]`;
        components: type extends keyof keyReferences ? Error<`Circular reference detected. '${typedDataParameter['type']}' is a circular reference.`> : _TypedDataParametersToAbiParameters<typedData[type], typedData, keyReferences & {
            [_ in type]: true;
        }>;
    } : typedDataParameter : never;
};
export {};
//# sourceMappingURL=utils.d.ts.map