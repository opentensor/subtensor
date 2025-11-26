import * as AbiError from 'ox/AbiError';
import { decodeErrorResult } from '../../../utils/abi/decodeErrorResult.js';
import { getContractError, } from '../../../utils/errors/getContractError.js';
import { FunctionSelectorNotRecognizedError, } from '../errors.js';
export function getExecuteError(e, parameters) {
    const error = e.walk((e) => 'data' in e);
    if (!error?.data)
        return e;
    if (error.data ===
        AbiError.getSelector(AbiError.from('error FnSelectorNotRecognized()')))
        return new FunctionSelectorNotRecognizedError();
    let matched = null;
    for (const c of parameters.calls) {
        const call = c;
        if (!call.abi)
            continue;
        try {
            const matches = Boolean(decodeErrorResult({
                abi: call.abi,
                data: error.data,
            }));
            if (!matches)
                continue;
            matched = call;
        }
        catch { }
    }
    if (matched)
        return getContractError(error, {
            abi: matched.abi,
            address: matched.to,
            args: matched.args,
            functionName: matched.functionName,
        });
    return e;
}
//# sourceMappingURL=getExecuteError.js.map