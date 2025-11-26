import * as abitype from 'abitype';
import * as AbiItem from './AbiItem.js';
import * as AbiParameters from './AbiParameters.js';
import * as Hex from './Hex.js';
// eslint-disable-next-line jsdoc/require-jsdoc
export function decode(...parameters) {
    const [abiConstructor, options] = (() => {
        if (Array.isArray(parameters[0])) {
            const [abi, options] = parameters;
            return [fromAbi(abi), options];
        }
        return parameters;
    })();
    const { bytecode } = options;
    if (abiConstructor.inputs.length === 0)
        return undefined;
    const data = options.data.replace(bytecode, '0x');
    return AbiParameters.decode(abiConstructor.inputs, data);
}
// eslint-disable-next-line jsdoc/require-jsdoc
export function encode(...parameters) {
    const [abiConstructor, options] = (() => {
        if (Array.isArray(parameters[0])) {
            const [abi, options] = parameters;
            return [fromAbi(abi), options];
        }
        return parameters;
    })();
    const { bytecode, args } = options;
    return Hex.concat(bytecode, abiConstructor.inputs?.length && args?.length
        ? AbiParameters.encode(abiConstructor.inputs, args)
        : '0x');
}
/** @internal */
export function format(abiConstructor) {
    return abitype.formatAbiItem(abiConstructor);
}
/** @internal */
export function from(abiConstructor) {
    return AbiItem.from(abiConstructor);
}
/** @internal */
export function fromAbi(abi) {
    const item = abi.find((item) => item.type === 'constructor');
    if (!item)
        throw new AbiItem.NotFoundError({ name: 'constructor' });
    return item;
}
//# sourceMappingURL=AbiConstructor.js.map