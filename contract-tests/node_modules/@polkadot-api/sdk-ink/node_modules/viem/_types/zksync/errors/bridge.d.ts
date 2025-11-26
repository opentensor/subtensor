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
export type CannotClaimSuccessfulDepositErrorType = CannotClaimSuccessfulDepositError & {
    name: 'CannotClaimSuccessfulDepositError';
};
export declare class CannotClaimSuccessfulDepositError extends BaseError {
    constructor({ hash }: {
        hash: Hash;
    });
}
export type LogProofNotFoundErrorType = LogProofNotFoundError & {
    name: 'LogProofNotFoundError';
};
export declare class LogProofNotFoundError extends BaseError {
    constructor({ hash, index }: {
        hash: Hash;
        index: number;
    });
}
export type L2BridgeNotFoundErrorType = L2BridgeNotFoundError & {
    name: 'L2BridgeNotFoundError';
};
export declare class L2BridgeNotFoundError extends BaseError {
    constructor();
}
//# sourceMappingURL=bridge.d.ts.map