import type { AbiItemType } from '../../abi.js';
import { BaseError } from '../../errors.js';
export declare class InvalidSignatureError extends BaseError {
    name: string;
    constructor({ signature, type, }: {
        signature: string;
        type: AbiItemType | 'struct';
    });
}
export declare class UnknownSignatureError extends BaseError {
    name: string;
    constructor({ signature }: {
        signature: string;
    });
}
export declare class InvalidStructSignatureError extends BaseError {
    name: string;
    constructor({ signature }: {
        signature: string;
    });
}
//# sourceMappingURL=signature.d.ts.map