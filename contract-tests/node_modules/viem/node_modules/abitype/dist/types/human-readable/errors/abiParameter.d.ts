import type { AbiItemType, AbiParameter } from '../../abi.js';
import { BaseError } from '../../errors.js';
import type { Modifier } from '../types/signatures.js';
export declare class InvalidAbiParameterError extends BaseError {
    name: string;
    constructor({ param }: {
        param: string | object;
    });
}
export declare class InvalidAbiParametersError extends BaseError {
    name: string;
    constructor({ params }: {
        params: string | object;
    });
}
export declare class InvalidParameterError extends BaseError {
    name: string;
    constructor({ param }: {
        param: string;
    });
}
export declare class SolidityProtectedKeywordError extends BaseError {
    name: string;
    constructor({ param, name }: {
        param: string;
        name: string;
    });
}
export declare class InvalidModifierError extends BaseError {
    name: string;
    constructor({ param, type, modifier, }: {
        param: string;
        type?: AbiItemType | 'struct' | undefined;
        modifier: Modifier;
    });
}
export declare class InvalidFunctionModifierError extends BaseError {
    name: string;
    constructor({ param, type, modifier, }: {
        param: string;
        type?: AbiItemType | 'struct' | undefined;
        modifier: Modifier;
    });
}
export declare class InvalidAbiTypeParameterError extends BaseError {
    name: string;
    constructor({ abiParameter, }: {
        abiParameter: AbiParameter & {
            indexed?: boolean | undefined;
        };
    });
}
//# sourceMappingURL=abiParameter.d.ts.map