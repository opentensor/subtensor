import { BaseError } from '../../errors/base.js';
export class BaseFeeHigherThanValueError extends BaseError {
    constructor(baseCost, value) {
        super([
            'The base cost of performing the priority operation is higher than the provided transaction value parameter.',
            '',
            `Base cost: ${baseCost}.`,
            `Provided value: ${value}.`,
        ].join('\n'), { name: 'BaseFeeHigherThanValueError' });
    }
}
export class TxHashNotFoundInLogsError extends BaseError {
    constructor() {
        super('The transaction hash not found in event logs.', {
            name: 'TxHashNotFoundInLogsError',
        });
    }
}
export class WithdrawalLogNotFoundError extends BaseError {
    constructor({ hash }) {
        super([
            `Withdrawal log with hash ${hash} not found.`,
            '',
            'Either the withdrawal transaction is still processing or it did not finish successfully.',
        ].join('\n'), { name: 'WithdrawalLogNotFoundError' });
    }
}
export class CannotClaimSuccessfulDepositError extends BaseError {
    constructor({ hash }) {
        super([`Cannot claim successful deposit: ${hash}.`].join('\n'), {
            name: 'CannotClaimSuccessfulDepositError',
        });
    }
}
export class LogProofNotFoundError extends BaseError {
    constructor({ hash, index }) {
        super([`Log proof not found for hash ${hash} and index ${index}.`].join('\n'), { name: 'LogProofNotFoundError' });
    }
}
export class L2BridgeNotFoundError extends BaseError {
    constructor() {
        super(['L2 bridge address not found.'].join('\n'), {
            name: 'L2BridgeNotFoundError',
        });
    }
}
//# sourceMappingURL=bridge.js.map