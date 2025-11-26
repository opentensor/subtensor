import { BaseError } from '../../errors.js';
export declare class InvalidAbiItemError extends BaseError {
    name: string;
    constructor({ signature }: {
        signature: string | object;
    });
}
export declare class UnknownTypeError extends BaseError {
    name: string;
    constructor({ type }: {
        type: string;
    });
}
export declare class UnknownSolidityTypeError extends BaseError {
    name: string;
    constructor({ type }: {
        type: string;
    });
}
//# sourceMappingURL=abiItem.d.ts.map