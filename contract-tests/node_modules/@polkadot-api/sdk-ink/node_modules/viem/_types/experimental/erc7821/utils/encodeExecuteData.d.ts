import type { Narrow } from 'abitype';
import type { ErrorType } from '../../../errors/utils.js';
import type { Calls } from '../../../types/calls.js';
import type { Hex } from '../../../types/misc.js';
import { type EncodeFunctionDataErrorType } from '../../../utils/abi/encodeFunctionData.js';
import { type EncodeCallsErrorType } from './encodeCalls.js';
export type EncodeExecuteDataParameters<calls extends readonly unknown[] = readonly unknown[]> = {
    /** Calls to execute. */
    calls: Calls<Narrow<calls>>;
    /** Additional data to include for execution. */
    opData?: Hex | undefined;
};
export type EncodeExecuteDataReturnType = Hex;
export type EncodeExecuteDataErrorType = EncodeCallsErrorType | EncodeFunctionDataErrorType | ErrorType;
export declare function encodeExecuteData<const calls extends readonly unknown[]>(parameters: EncodeExecuteDataParameters<calls>): EncodeExecuteDataReturnType;
//# sourceMappingURL=encodeExecuteData.d.ts.map