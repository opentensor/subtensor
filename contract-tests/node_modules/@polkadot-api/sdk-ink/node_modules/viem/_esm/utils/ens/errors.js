import { BaseError } from '../../errors/base.js';
import { ContractFunctionRevertedError } from '../../errors/contract.js';
/*
 * @description Checks if error is a valid null result UniversalResolver error
 */
export function isNullUniversalResolverError(err) {
    if (!(err instanceof BaseError))
        return false;
    const cause = err.walk((e) => e instanceof ContractFunctionRevertedError);
    if (!(cause instanceof ContractFunctionRevertedError))
        return false;
    if (cause.data?.errorName === 'HttpError')
        return true;
    if (cause.data?.errorName === 'ResolverError')
        return true;
    if (cause.data?.errorName === 'ResolverNotContract')
        return true;
    if (cause.data?.errorName === 'ResolverNotFound')
        return true;
    if (cause.data?.errorName === 'ReverseAddressMismatch')
        return true;
    if (cause.data?.errorName === 'UnsupportedResolverProfile')
        return true;
    return false;
}
//# sourceMappingURL=errors.js.map