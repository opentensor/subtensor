import * as AbiParameters from 'ox/AbiParameters';
import type { ErrorType } from '../../../errors/utils.js';
import type { Calls } from '../../../types/calls.js';
import type { Hex } from '../../../types/misc.js';
import { type EncodeFunctionDataErrorType } from '../../../utils/abi/encodeFunctionData.js';
export type EncodeCallsErrorType = AbiParameters.encode.ErrorType | AbiParameters.from.ErrorType | EncodeFunctionDataErrorType | ErrorType;
export declare function encodeCalls(calls_: Calls<readonly unknown[]>, opData?: Hex | undefined): `0x${string}`;
//# sourceMappingURL=encodeCalls.d.ts.map