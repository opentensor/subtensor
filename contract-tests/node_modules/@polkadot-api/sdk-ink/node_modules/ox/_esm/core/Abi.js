import * as abitype from 'abitype';
import * as internal from './internal/abi.js';
// eslint-disable-next-line jsdoc/require-jsdoc
export function format(abi) {
    return abitype.formatAbi(abi);
}
// eslint-disable-next-line jsdoc/require-jsdoc
export function from(abi) {
    if (internal.isSignatures(abi))
        return abitype.parseAbi(abi);
    return abi;
}
//# sourceMappingURL=Abi.js.map