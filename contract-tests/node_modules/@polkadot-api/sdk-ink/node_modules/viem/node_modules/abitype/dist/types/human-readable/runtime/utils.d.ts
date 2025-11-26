import type { AbiItemType, AbiType, SolidityArray, SolidityBytes, SolidityString, SolidityTuple } from '../../abi.js';
import type { Modifier } from '../types/signatures.js';
import type { StructLookup } from '../types/structs.js';
export declare function parseSignature(signature: string, structs?: StructLookup): {
    name: string;
    type: string;
    inputs: (({
        type: string;
        name?: string | undefined | undefined;
        internalType?: string | undefined;
    } & {
        indexed?: boolean;
    }) | {
        indexed: boolean;
        name: string;
        type: string;
    } | {
        indexed?: never;
        name: string;
        type: string;
    } | {
        indexed: boolean;
        name?: never;
        type: string;
    } | {
        indexed?: never;
        name?: never;
        type: string;
    })[];
} | {
    type: string;
    stateMutability: string;
};
export declare function parseFunctionSignature(signature: string, structs?: StructLookup): {
    name: string;
    type: string;
    stateMutability: import("../../abi.js").AbiStateMutability;
    inputs: (({
        type: string;
        name?: string | undefined | undefined;
        internalType?: string | undefined;
    } & {
        indexed?: boolean;
    }) | {
        indexed: boolean;
        name: string;
        type: string;
    } | {
        indexed?: never;
        name: string;
        type: string;
    } | {
        indexed: boolean;
        name?: never;
        type: string;
    } | {
        indexed?: never;
        name?: never;
        type: string;
    })[];
    outputs: (({
        type: string;
        name?: string | undefined | undefined;
        internalType?: string | undefined;
    } & {
        indexed?: boolean;
    }) | {
        indexed: boolean;
        name: string;
        type: string;
    } | {
        indexed?: never;
        name: string;
        type: string;
    } | {
        indexed: boolean;
        name?: never;
        type: string;
    } | {
        indexed?: never;
        name?: never;
        type: string;
    })[];
};
export declare function parseEventSignature(signature: string, structs?: StructLookup): {
    name: string;
    type: string;
    inputs: (({
        type: string;
        name?: string | undefined | undefined;
        internalType?: string | undefined;
    } & {
        indexed?: boolean;
    }) | {
        indexed: boolean;
        name: string;
        type: string;
    } | {
        indexed?: never;
        name: string;
        type: string;
    } | {
        indexed: boolean;
        name?: never;
        type: string;
    } | {
        indexed?: never;
        name?: never;
        type: string;
    })[];
};
export declare function parseErrorSignature(signature: string, structs?: StructLookup): {
    name: string;
    type: string;
    inputs: (({
        type: string;
        name?: string | undefined | undefined;
        internalType?: string | undefined;
    } & {
        indexed?: boolean;
    }) | {
        indexed: boolean;
        name: string;
        type: string;
    } | {
        indexed?: never;
        name: string;
        type: string;
    } | {
        indexed: boolean;
        name?: never;
        type: string;
    } | {
        indexed?: never;
        name?: never;
        type: string;
    })[];
};
export declare function parseConstructorSignature(signature: string, structs?: StructLookup): {
    type: string;
    stateMutability: string;
    inputs: (({
        type: string;
        name?: string | undefined | undefined;
        internalType?: string | undefined;
    } & {
        indexed?: boolean;
    }) | {
        indexed: boolean;
        name: string;
        type: string;
    } | {
        indexed?: never;
        name: string;
        type: string;
    } | {
        indexed: boolean;
        name?: never;
        type: string;
    } | {
        indexed?: never;
        name?: never;
        type: string;
    })[];
};
export declare function parseFallbackSignature(signature: string): {
    type: string;
    stateMutability: string;
};
type ParseOptions = {
    modifiers?: Set<Modifier>;
    structs?: StructLookup;
    type?: AbiItemType | 'struct';
};
export declare function parseAbiParameter(param: string, options?: ParseOptions): ({
    type: string;
    name?: string | undefined | undefined;
    internalType?: string | undefined;
} & {
    indexed?: boolean;
}) | {
    indexed: boolean;
    name: string;
    type: string;
} | {
    indexed?: never;
    name: string;
    type: string;
} | {
    indexed: boolean;
    name?: never;
    type: string;
} | {
    indexed?: never;
    name?: never;
    type: string;
};
export declare function splitParameters(params: string, result?: string[], current?: string, depth?: number): readonly string[];
export declare function isSolidityType(type: string): type is Exclude<AbiType, SolidityTuple | SolidityArray>;
/** @internal */
export declare function isSolidityKeyword(name: string): boolean;
/** @internal */
export declare function isValidDataLocation(type: string, isArray: boolean): type is Exclude<AbiType, SolidityString | Extract<SolidityBytes, 'bytes'> | SolidityArray>;
export {};
//# sourceMappingURL=utils.d.ts.map