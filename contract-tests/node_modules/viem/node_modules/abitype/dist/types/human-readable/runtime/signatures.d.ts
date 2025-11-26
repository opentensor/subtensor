import type { AbiStateMutability } from '../../abi.js';
import type { FunctionModifier, Modifier } from '../types/signatures.js';
export declare function isErrorSignature(signature: string): boolean;
export declare function execErrorSignature(signature: string): {
    name: string;
    parameters: string;
} | undefined;
export declare function isEventSignature(signature: string): boolean;
export declare function execEventSignature(signature: string): {
    name: string;
    parameters: string;
} | undefined;
export declare function isFunctionSignature(signature: string): boolean;
export declare function execFunctionSignature(signature: string): {
    name: string;
    parameters: string;
    stateMutability?: AbiStateMutability;
    returns?: string;
} | undefined;
export declare function isStructSignature(signature: string): boolean;
export declare function execStructSignature(signature: string): {
    name: string;
    properties: string;
} | undefined;
export declare function isConstructorSignature(signature: string): boolean;
export declare function execConstructorSignature(signature: string): {
    parameters: string;
    stateMutability?: Extract<AbiStateMutability, "payable">;
} | undefined;
export declare function isFallbackSignature(signature: string): boolean;
export declare function execFallbackSignature(signature: string): {
    parameters: string;
    stateMutability?: Extract<AbiStateMutability, "payable">;
} | undefined;
export declare function isReceiveSignature(signature: string): boolean;
export declare const modifiers: Set<Modifier>;
export declare const eventModifiers: Set<"indexed">;
export declare const functionModifiers: Set<FunctionModifier>;
//# sourceMappingURL=signatures.d.ts.map