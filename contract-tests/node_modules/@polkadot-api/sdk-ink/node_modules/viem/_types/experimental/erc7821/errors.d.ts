import { BaseError } from '../../errors/base.js';
export type ExecuteUnsupportedErrorType = ExecuteUnsupportedError & {
    name: 'ExecuteUnsupportedError';
};
export declare class ExecuteUnsupportedError extends BaseError {
    constructor();
}
export type FunctionSelectorNotRecognizedErrorType = FunctionSelectorNotRecognizedError & {
    name: 'FunctionSelectorNotRecognizedError';
};
export declare class FunctionSelectorNotRecognizedError extends BaseError {
    constructor();
}
//# sourceMappingURL=errors.d.ts.map