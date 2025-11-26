import { z } from 'zod';
import type { AbiEventParameter as AbiEventParameterType, AbiParameter as AbiParameterType } from './abi.js';
export declare const Address: z.ZodEffects<z.ZodString, `0x${string}`, string>;
export declare const SolidityAddress: z.ZodLiteral<"address">;
export declare const SolidityBool: z.ZodLiteral<"bool">;
export declare const SolidityBytes: z.ZodString;
export declare const SolidityFunction: z.ZodLiteral<"function">;
export declare const SolidityString: z.ZodLiteral<"string">;
export declare const SolidityTuple: z.ZodLiteral<"tuple">;
export declare const SolidityInt: z.ZodString;
export declare const SolidityArrayWithoutTuple: z.ZodString;
export declare const SolidityArrayWithTuple: z.ZodString;
export declare const SolidityArray: z.ZodUnion<[z.ZodString, z.ZodString]>;
export declare const AbiParameter: z.ZodType<AbiParameterType>;
export declare const AbiEventParameter: z.ZodType<AbiEventParameterType>;
export declare const AbiStateMutability: z.ZodUnion<[z.ZodLiteral<"pure">, z.ZodLiteral<"view">, z.ZodLiteral<"nonpayable">, z.ZodLiteral<"payable">]>;
export declare const AbiFunction: z.ZodEffects<z.ZodObject<{
    type: z.ZodLiteral<"function">;
    /**
     * @deprecated use `pure` or `view` from {@link AbiStateMutability} instead
     * https://github.com/ethereum/solidity/issues/992
     */
    constant: z.ZodOptional<z.ZodBoolean>;
    /**
     * @deprecated Vyper used to provide gas estimates
     * https://github.com/vyperlang/vyper/issues/2151
     */
    gas: z.ZodOptional<z.ZodNumber>;
    inputs: z.ZodReadonly<z.ZodArray<z.ZodType<AbiParameterType, z.ZodTypeDef, AbiParameterType>, "many">>;
    name: z.ZodString;
    outputs: z.ZodReadonly<z.ZodArray<z.ZodType<AbiParameterType, z.ZodTypeDef, AbiParameterType>, "many">>;
    /**
     * @deprecated use `payable` or `nonpayable` from {@link AbiStateMutability} instead
     * https://github.com/ethereum/solidity/issues/992
     */
    payable: z.ZodOptional<z.ZodBoolean>;
    stateMutability: z.ZodUnion<[z.ZodLiteral<"pure">, z.ZodLiteral<"view">, z.ZodLiteral<"nonpayable">, z.ZodLiteral<"payable">]>;
}, "strip", z.ZodTypeAny, {
    inputs: readonly AbiParameterType[];
    outputs: readonly AbiParameterType[];
    type: "function";
    name: string;
    stateMutability: "pure" | "view" | "nonpayable" | "payable";
    payable?: boolean | undefined;
    constant?: boolean | undefined;
    gas?: number | undefined;
}, {
    inputs: readonly AbiParameterType[];
    outputs: readonly AbiParameterType[];
    type: "function";
    name: string;
    stateMutability: "pure" | "view" | "nonpayable" | "payable";
    payable?: boolean | undefined;
    constant?: boolean | undefined;
    gas?: number | undefined;
}>, {
    inputs: readonly AbiParameterType[];
    outputs: readonly AbiParameterType[];
    type: "function";
    name: string;
    stateMutability: "pure" | "view" | "nonpayable" | "payable";
    payable?: boolean | undefined;
    constant?: boolean | undefined;
    gas?: number | undefined;
}, unknown>;
export declare const AbiConstructor: z.ZodEffects<z.ZodObject<{
    type: z.ZodLiteral<"constructor">;
    /**
     * @deprecated use `pure` or `view` from {@link AbiStateMutability} instead
     * https://github.com/ethereum/solidity/issues/992
     */
    inputs: z.ZodReadonly<z.ZodArray<z.ZodType<AbiParameterType, z.ZodTypeDef, AbiParameterType>, "many">>;
    /**
     * @deprecated use `payable` or `nonpayable` from {@link AbiStateMutability} instead
     * https://github.com/ethereum/solidity/issues/992
     */
    payable: z.ZodOptional<z.ZodBoolean>;
    stateMutability: z.ZodUnion<[z.ZodLiteral<"nonpayable">, z.ZodLiteral<"payable">]>;
}, "strip", z.ZodTypeAny, {
    inputs: readonly AbiParameterType[];
    type: "constructor";
    stateMutability: "nonpayable" | "payable";
    payable?: boolean | undefined;
}, {
    inputs: readonly AbiParameterType[];
    type: "constructor";
    stateMutability: "nonpayable" | "payable";
    payable?: boolean | undefined;
}>, {
    inputs: readonly AbiParameterType[];
    type: "constructor";
    stateMutability: "nonpayable" | "payable";
    payable?: boolean | undefined;
}, unknown>;
export declare const AbiFallback: z.ZodEffects<z.ZodObject<{
    type: z.ZodLiteral<"fallback">;
    /**
     * @deprecated use `payable` or `nonpayable` from {@link AbiStateMutability} instead
     * https://github.com/ethereum/solidity/issues/992
     */
    payable: z.ZodOptional<z.ZodBoolean>;
    stateMutability: z.ZodUnion<[z.ZodLiteral<"nonpayable">, z.ZodLiteral<"payable">]>;
}, "strip", z.ZodTypeAny, {
    type: "fallback";
    stateMutability: "nonpayable" | "payable";
    payable?: boolean | undefined;
}, {
    type: "fallback";
    stateMutability: "nonpayable" | "payable";
    payable?: boolean | undefined;
}>, {
    type: "fallback";
    stateMutability: "nonpayable" | "payable";
    payable?: boolean | undefined;
}, unknown>;
export declare const AbiReceive: z.ZodObject<{
    type: z.ZodLiteral<"receive">;
    stateMutability: z.ZodLiteral<"payable">;
}, "strip", z.ZodTypeAny, {
    type: "receive";
    stateMutability: "payable";
}, {
    type: "receive";
    stateMutability: "payable";
}>;
export declare const AbiEvent: z.ZodObject<{
    type: z.ZodLiteral<"event">;
    anonymous: z.ZodOptional<z.ZodBoolean>;
    inputs: z.ZodReadonly<z.ZodArray<z.ZodType<AbiEventParameterType, z.ZodTypeDef, AbiEventParameterType>, "many">>;
    name: z.ZodString;
}, "strip", z.ZodTypeAny, {
    inputs: readonly AbiEventParameterType[];
    type: "event";
    name: string;
    anonymous?: boolean | undefined;
}, {
    inputs: readonly AbiEventParameterType[];
    type: "event";
    name: string;
    anonymous?: boolean | undefined;
}>;
export declare const AbiError: z.ZodObject<{
    type: z.ZodLiteral<"error">;
    inputs: z.ZodReadonly<z.ZodArray<z.ZodType<AbiParameterType, z.ZodTypeDef, AbiParameterType>, "many">>;
    name: z.ZodString;
}, "strip", z.ZodTypeAny, {
    inputs: readonly AbiParameterType[];
    type: "error";
    name: string;
}, {
    inputs: readonly AbiParameterType[];
    type: "error";
    name: string;
}>;
export declare const AbiItemType: z.ZodUnion<[z.ZodLiteral<"constructor">, z.ZodLiteral<"event">, z.ZodLiteral<"error">, z.ZodLiteral<"fallback">, z.ZodLiteral<"function">, z.ZodLiteral<"receive">]>;
/**
 * Zod Schema for Contract [ABI Specification](https://docs.soliditylang.org/en/latest/abi-spec.html#json)
 *
 * @example
 * const parsedAbi = Abi.parse([…])
 */
export declare const Abi: z.ZodReadonly<z.ZodArray<z.ZodUnion<[z.ZodObject<{
    type: z.ZodLiteral<"error">;
    inputs: z.ZodReadonly<z.ZodArray<z.ZodType<AbiParameterType, z.ZodTypeDef, AbiParameterType>, "many">>;
    name: z.ZodString;
}, "strip", z.ZodTypeAny, {
    inputs: readonly AbiParameterType[];
    type: "error";
    name: string;
}, {
    inputs: readonly AbiParameterType[];
    type: "error";
    name: string;
}>, z.ZodObject<{
    type: z.ZodLiteral<"event">;
    anonymous: z.ZodOptional<z.ZodBoolean>;
    inputs: z.ZodReadonly<z.ZodArray<z.ZodType<AbiEventParameterType, z.ZodTypeDef, AbiEventParameterType>, "many">>;
    name: z.ZodString;
}, "strip", z.ZodTypeAny, {
    inputs: readonly AbiEventParameterType[];
    type: "event";
    name: string;
    anonymous?: boolean | undefined;
}, {
    inputs: readonly AbiEventParameterType[];
    type: "event";
    name: string;
    anonymous?: boolean | undefined;
}>, z.ZodEffects<z.ZodIntersection<z.ZodObject<{
    /**
     * @deprecated use `pure` or `view` from {@link AbiStateMutability} instead
     * https://github.com/ethereum/solidity/issues/992
     */
    constant: z.ZodOptional<z.ZodBoolean>;
    /**
     * @deprecated Vyper used to provide gas estimates
     * https://github.com/vyperlang/vyper/issues/2151
     */
    gas: z.ZodOptional<z.ZodNumber>;
    /**
     * @deprecated use `payable` or `nonpayable` from {@link AbiStateMutability} instead
     * https://github.com/ethereum/solidity/issues/992
     */
    payable: z.ZodOptional<z.ZodBoolean>;
}, "strip", z.ZodTypeAny, {
    payable?: boolean | undefined;
    constant?: boolean | undefined;
    gas?: number | undefined;
}, {
    payable?: boolean | undefined;
    constant?: boolean | undefined;
    gas?: number | undefined;
}>, z.ZodDiscriminatedUnion<"type", [z.ZodObject<{
    type: z.ZodLiteral<"function">;
    inputs: z.ZodReadonly<z.ZodArray<z.ZodType<AbiParameterType, z.ZodTypeDef, AbiParameterType>, "many">>;
    name: z.ZodString;
    outputs: z.ZodReadonly<z.ZodArray<z.ZodType<AbiParameterType, z.ZodTypeDef, AbiParameterType>, "many">>;
    stateMutability: z.ZodUnion<[z.ZodLiteral<"pure">, z.ZodLiteral<"view">, z.ZodLiteral<"nonpayable">, z.ZodLiteral<"payable">]>;
}, "strip", z.ZodTypeAny, {
    inputs: readonly AbiParameterType[];
    outputs: readonly AbiParameterType[];
    type: "function";
    name: string;
    stateMutability: "pure" | "view" | "nonpayable" | "payable";
}, {
    inputs: readonly AbiParameterType[];
    outputs: readonly AbiParameterType[];
    type: "function";
    name: string;
    stateMutability: "pure" | "view" | "nonpayable" | "payable";
}>, z.ZodObject<{
    type: z.ZodLiteral<"constructor">;
    inputs: z.ZodReadonly<z.ZodArray<z.ZodType<AbiParameterType, z.ZodTypeDef, AbiParameterType>, "many">>;
    stateMutability: z.ZodUnion<[z.ZodLiteral<"payable">, z.ZodLiteral<"nonpayable">]>;
}, "strip", z.ZodTypeAny, {
    inputs: readonly AbiParameterType[];
    type: "constructor";
    stateMutability: "nonpayable" | "payable";
}, {
    inputs: readonly AbiParameterType[];
    type: "constructor";
    stateMutability: "nonpayable" | "payable";
}>, z.ZodObject<{
    type: z.ZodLiteral<"fallback">;
    inputs: z.ZodOptional<z.ZodTuple<[], null>>;
    stateMutability: z.ZodUnion<[z.ZodLiteral<"payable">, z.ZodLiteral<"nonpayable">]>;
}, "strip", z.ZodTypeAny, {
    type: "fallback";
    stateMutability: "nonpayable" | "payable";
    inputs?: [] | undefined;
}, {
    type: "fallback";
    stateMutability: "nonpayable" | "payable";
    inputs?: [] | undefined;
}>, z.ZodObject<{
    type: z.ZodLiteral<"receive">;
    stateMutability: z.ZodLiteral<"payable">;
}, "strip", z.ZodTypeAny, {
    type: "receive";
    stateMutability: "payable";
}, {
    type: "receive";
    stateMutability: "payable";
}>]>>, {
    payable?: boolean | undefined;
    constant?: boolean | undefined;
    gas?: number | undefined;
} & ({
    inputs: readonly AbiParameterType[];
    outputs: readonly AbiParameterType[];
    type: "function";
    name: string;
    stateMutability: "pure" | "view" | "nonpayable" | "payable";
} | {
    inputs: readonly AbiParameterType[];
    type: "constructor";
    stateMutability: "nonpayable" | "payable";
} | {
    type: "fallback";
    stateMutability: "nonpayable" | "payable";
    inputs?: [] | undefined;
} | {
    type: "receive";
    stateMutability: "payable";
}), unknown>]>, "many">>;
export declare const TypedDataDomain: z.ZodObject<{
    chainId: z.ZodOptional<z.ZodUnion<[z.ZodNumber, z.ZodBigInt]>>;
    name: z.ZodOptional<z.ZodString>;
    salt: z.ZodOptional<z.ZodString>;
    verifyingContract: z.ZodOptional<z.ZodEffects<z.ZodString, `0x${string}`, string>>;
    version: z.ZodOptional<z.ZodString>;
}, "strip", z.ZodTypeAny, {
    name?: string | undefined;
    chainId?: number | bigint | undefined;
    salt?: string | undefined;
    verifyingContract?: `0x${string}` | undefined;
    version?: string | undefined;
}, {
    name?: string | undefined;
    chainId?: number | bigint | undefined;
    salt?: string | undefined;
    verifyingContract?: string | undefined;
    version?: string | undefined;
}>;
export declare const TypedDataType: z.ZodUnion<[z.ZodLiteral<"address">, z.ZodLiteral<"bool">, z.ZodString, z.ZodLiteral<"string">, z.ZodString, z.ZodUnion<[z.ZodString, z.ZodString]>]>;
export declare const TypedDataParameter: z.ZodObject<{
    name: z.ZodString;
    type: z.ZodString;
}, "strip", z.ZodTypeAny, {
    type: string;
    name: string;
}, {
    type: string;
    name: string;
}>;
export declare const TypedData: z.ZodEffects<z.ZodRecord<z.ZodString, z.ZodArray<z.ZodObject<{
    name: z.ZodString;
    type: z.ZodString;
}, "strip", z.ZodTypeAny, {
    type: string;
    name: string;
}, {
    type: string;
    name: string;
}>, "many">>, {
    [x: string]: readonly import("./abi.js").TypedDataParameter[];
    [x: `string[${string}]`]: undefined;
    [x: `function[${string}]`]: undefined;
    [x: `bytes[${string}]`]: undefined;
    [x: `bytes1[${string}]`]: undefined;
    [x: `bytes2[${string}]`]: undefined;
    [x: `bytes3[${string}]`]: undefined;
    [x: `bytes4[${string}]`]: undefined;
    [x: `bytes5[${string}]`]: undefined;
    [x: `bytes6[${string}]`]: undefined;
    [x: `bytes7[${string}]`]: undefined;
    [x: `bytes8[${string}]`]: undefined;
    [x: `bytes9[${string}]`]: undefined;
    [x: `bytes10[${string}]`]: undefined;
    [x: `bytes11[${string}]`]: undefined;
    [x: `bytes12[${string}]`]: undefined;
    [x: `bytes13[${string}]`]: undefined;
    [x: `bytes14[${string}]`]: undefined;
    [x: `bytes15[${string}]`]: undefined;
    [x: `bytes16[${string}]`]: undefined;
    [x: `bytes17[${string}]`]: undefined;
    [x: `bytes18[${string}]`]: undefined;
    [x: `bytes19[${string}]`]: undefined;
    [x: `bytes20[${string}]`]: undefined;
    [x: `bytes21[${string}]`]: undefined;
    [x: `bytes22[${string}]`]: undefined;
    [x: `bytes23[${string}]`]: undefined;
    [x: `bytes24[${string}]`]: undefined;
    [x: `bytes25[${string}]`]: undefined;
    [x: `bytes26[${string}]`]: undefined;
    [x: `bytes27[${string}]`]: undefined;
    [x: `bytes28[${string}]`]: undefined;
    [x: `bytes29[${string}]`]: undefined;
    [x: `bytes30[${string}]`]: undefined;
    [x: `bytes31[${string}]`]: undefined;
    [x: `bytes32[${string}]`]: undefined;
    [x: `int[${string}]`]: undefined;
    [x: `int8[${string}]`]: undefined;
    [x: `int16[${string}]`]: undefined;
    [x: `int24[${string}]`]: undefined;
    [x: `int32[${string}]`]: undefined;
    [x: `int40[${string}]`]: undefined;
    [x: `int48[${string}]`]: undefined;
    [x: `int56[${string}]`]: undefined;
    [x: `int64[${string}]`]: undefined;
    [x: `int72[${string}]`]: undefined;
    [x: `int80[${string}]`]: undefined;
    [x: `int88[${string}]`]: undefined;
    [x: `int96[${string}]`]: undefined;
    [x: `int104[${string}]`]: undefined;
    [x: `int112[${string}]`]: undefined;
    [x: `int120[${string}]`]: undefined;
    [x: `int128[${string}]`]: undefined;
    [x: `int136[${string}]`]: undefined;
    [x: `int144[${string}]`]: undefined;
    [x: `int152[${string}]`]: undefined;
    [x: `int160[${string}]`]: undefined;
    [x: `int168[${string}]`]: undefined;
    [x: `int176[${string}]`]: undefined;
    [x: `int184[${string}]`]: undefined;
    [x: `int192[${string}]`]: undefined;
    [x: `int200[${string}]`]: undefined;
    [x: `int208[${string}]`]: undefined;
    [x: `int216[${string}]`]: undefined;
    [x: `int224[${string}]`]: undefined;
    [x: `int232[${string}]`]: undefined;
    [x: `int240[${string}]`]: undefined;
    [x: `int248[${string}]`]: undefined;
    [x: `int256[${string}]`]: undefined;
    [x: `uint[${string}]`]: undefined;
    [x: `uint8[${string}]`]: undefined;
    [x: `uint16[${string}]`]: undefined;
    [x: `uint24[${string}]`]: undefined;
    [x: `uint32[${string}]`]: undefined;
    [x: `uint40[${string}]`]: undefined;
    [x: `uint48[${string}]`]: undefined;
    [x: `uint56[${string}]`]: undefined;
    [x: `uint64[${string}]`]: undefined;
    [x: `uint72[${string}]`]: undefined;
    [x: `uint80[${string}]`]: undefined;
    [x: `uint88[${string}]`]: undefined;
    [x: `uint96[${string}]`]: undefined;
    [x: `uint104[${string}]`]: undefined;
    [x: `uint112[${string}]`]: undefined;
    [x: `uint120[${string}]`]: undefined;
    [x: `uint128[${string}]`]: undefined;
    [x: `uint136[${string}]`]: undefined;
    [x: `uint144[${string}]`]: undefined;
    [x: `uint152[${string}]`]: undefined;
    [x: `uint160[${string}]`]: undefined;
    [x: `uint168[${string}]`]: undefined;
    [x: `uint176[${string}]`]: undefined;
    [x: `uint184[${string}]`]: undefined;
    [x: `uint192[${string}]`]: undefined;
    [x: `uint200[${string}]`]: undefined;
    [x: `uint208[${string}]`]: undefined;
    [x: `uint216[${string}]`]: undefined;
    [x: `uint224[${string}]`]: undefined;
    [x: `uint232[${string}]`]: undefined;
    [x: `uint240[${string}]`]: undefined;
    [x: `uint248[${string}]`]: undefined;
    [x: `uint256[${string}]`]: undefined;
    [x: `address[${string}]`]: undefined;
    [x: `bool[${string}]`]: undefined;
    string?: never;
    bytes?: never;
    bytes1?: never;
    bytes2?: never;
    bytes3?: never;
    bytes4?: never;
    bytes5?: never;
    bytes6?: never;
    bytes7?: never;
    bytes8?: never;
    bytes9?: never;
    bytes10?: never;
    bytes11?: never;
    bytes12?: never;
    bytes13?: never;
    bytes14?: never;
    bytes15?: never;
    bytes16?: never;
    bytes17?: never;
    bytes18?: never;
    bytes19?: never;
    bytes20?: never;
    bytes21?: never;
    bytes22?: never;
    bytes23?: never;
    bytes24?: never;
    bytes25?: never;
    bytes26?: never;
    bytes27?: never;
    bytes28?: never;
    bytes29?: never;
    bytes30?: never;
    bytes31?: never;
    bytes32?: never;
    int8?: never;
    int16?: never;
    int24?: never;
    int32?: never;
    int40?: never;
    int48?: never;
    int56?: never;
    int64?: never;
    int72?: never;
    int80?: never;
    int88?: never;
    int96?: never;
    int104?: never;
    int112?: never;
    int120?: never;
    int128?: never;
    int136?: never;
    int144?: never;
    int152?: never;
    int160?: never;
    int168?: never;
    int176?: never;
    int184?: never;
    int192?: never;
    int200?: never;
    int208?: never;
    int216?: never;
    int224?: never;
    int232?: never;
    int240?: never;
    int248?: never;
    int256?: never;
    uint8?: never;
    uint16?: never;
    uint24?: never;
    uint32?: never;
    uint40?: never;
    uint48?: never;
    uint56?: never;
    uint64?: never;
    uint72?: never;
    uint80?: never;
    uint88?: never;
    uint96?: never;
    uint104?: never;
    uint112?: never;
    uint120?: never;
    uint128?: never;
    uint136?: never;
    uint144?: never;
    uint152?: never;
    uint160?: never;
    uint168?: never;
    uint176?: never;
    uint184?: never;
    uint192?: never;
    uint200?: never;
    uint208?: never;
    uint216?: never;
    uint224?: never;
    uint232?: never;
    uint240?: never;
    uint248?: never;
    uint256?: never;
    address?: never;
    bool?: never;
}, Record<string, {
    type: string;
    name: string;
}[]>>;
//# sourceMappingURL=zod.d.ts.map