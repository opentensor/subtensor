import type { Narrow } from 'abitype';
import type { ErrorType } from '../../../errors/utils.js';
import type { Batches } from '../../../types/calls.js';
import type { Hex } from '../../../types/misc.js';
import { type EncodeFunctionDataErrorType } from '../../../utils/abi/encodeFunctionData.js';
import { type EncodeCallsErrorType } from './encodeCalls.js';
/** @internal */
export type Batch = {
    calls: readonly unknown[];
    opData?: Hex | undefined;
};
export type EncodeExecuteBatchesDataParameters<batches extends readonly Batch[] = readonly Batch[]> = {
    /** Batches to execute. */
    batches: Batches<Narrow<batches>, {
        opData?: Hex | undefined;
    }>;
};
export type EncodeExecuteBatchesDataReturnType = Hex;
export type EncodeExecuteBatchesDataErrorType = EncodeCallsErrorType | EncodeFunctionDataErrorType | ErrorType;
export declare function encodeExecuteBatchesData<batches extends readonly Batch[]>(parameters: EncodeExecuteBatchesDataParameters<batches>): EncodeExecuteBatchesDataReturnType;
//# sourceMappingURL=encodeExecuteBatchesData.d.ts.map