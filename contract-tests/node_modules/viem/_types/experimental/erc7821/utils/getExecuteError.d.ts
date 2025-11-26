import type { Narrow } from 'abitype';
import type { BaseError } from '../../../errors/base.js';
import type { Calls } from '../../../types/calls.js';
import { type GetContractErrorReturnType } from '../../../utils/errors/getContractError.js';
import { type FunctionSelectorNotRecognizedErrorType } from '../errors.js';
export type GetExecuteErrorParameters<calls extends readonly unknown[] = readonly unknown[]> = {
    /** Calls to execute. */
    calls: Calls<Narrow<calls>>;
};
export type GetExecuteErrorReturnType = FunctionSelectorNotRecognizedErrorType | GetContractErrorReturnType;
export declare function getExecuteError<const calls extends readonly unknown[]>(e: BaseError, parameters: GetExecuteErrorParameters<calls>): GetExecuteErrorReturnType;
//# sourceMappingURL=getExecuteError.d.ts.map