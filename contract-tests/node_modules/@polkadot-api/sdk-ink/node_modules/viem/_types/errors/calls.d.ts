import type { GetCallsStatusReturnType } from '../actions/wallet/getCallsStatus.js';
import { BaseError } from './base.js';
export type BundleFailedErrorType = BundleFailedError & {
    name: 'BundleFailedError';
};
export declare class BundleFailedError extends BaseError {
    result: GetCallsStatusReturnType;
    constructor(result: GetCallsStatusReturnType);
}
//# sourceMappingURL=calls.d.ts.map