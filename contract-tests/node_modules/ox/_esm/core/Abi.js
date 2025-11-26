import * as abitype from 'abitype';
import * as internal from './internal/abi.js';
/** @internal */
export function format(abi) {
    return abitype.formatAbi(abi);
}
/** @internal */
export function from(abi) {
    if (internal.isSignatures(abi))
        return abitype.parseAbi(abi);
    return abi;
}
//# sourceMappingURL=Abi.js.map