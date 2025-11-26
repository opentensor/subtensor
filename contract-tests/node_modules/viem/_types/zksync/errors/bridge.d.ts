import { BaseError } from '../../errors/base.js';
import type { Hash } from '../../types/misc.js';
export type BaseFeeHigherThanValueErrorType = BaseFeeHigherThanValueError & {
    name: 'BaseFeeHigherThanValueError';
};
export declare class BaseFeeHigherThanValueError extends BaseError {
    constructor(baseCost: bigint, value: bigint);
}
export type TxHashNotFoundInLogsErrorType = BaseFeeHigherThanValueError & {
    name: 'TxHashNotFoundInLogsError';
};
export declare class TxHashNotFoundInLogsError extends BaseError {
    constructor();
}
export type WithdrawalLogNotFoundErrorType = WithdrawalLogNotFoundError & {
    name: 'WithdrawalLogNotFoundError';
};
export declare class WithdrawalLogNotFoundError extends BaseError {
    constructor({ hash }: {
        hash: Hash;
    });
}
//# sourceMappingURL=bridge.d.ts.map