"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.decodeData = decodeData;
exports.decodeResult = decodeResult;
exports.encodeData = encodeData;
exports.encodeResult = encodeResult;
exports.format = format;
exports.from = from;
exports.fromAbi = fromAbi;
exports.getSelector = getSelector;
const abitype = require("abitype");
const AbiItem = require("./AbiItem.js");
const AbiParameters = require("./AbiParameters.js");
const Hex = require("./Hex.js");
function decodeData(...parameters) {
    const [abiFunction, data] = (() => {
        if (Array.isArray(parameters[0])) {
            const [abi, name, data] = parameters;
            return [fromAbi(abi, name), data];
        }
        return parameters;
    })();
    const { overloads } = abiFunction;
    if (Hex.size(data) < 4)
        throw new AbiItem.InvalidSelectorSizeError({ data });
    if (abiFunction.inputs.length === 0)
        return undefined;
    const item = overloads
        ? fromAbi([abiFunction, ...overloads], data)
        : abiFunction;
    if (Hex.size(data) <= 4)
        return undefined;
    return AbiParameters.decode(item.inputs, Hex.slice(data, 4));
}
function decodeResult(...parameters) {
    const [abiFunction, data, options = {}] = (() => {
        if (Array.isArray(parameters[0])) {
            const [abi, name, data, options] = parameters;
            return [fromAbi(abi, name), data, options];
        }
        return parameters;
    })();
    const values = AbiParameters.decode(abiFunction.outputs, data, options);
    if (values && Object.keys(values).length === 0)
        return undefined;
    if (values && Object.keys(values).length === 1) {
        if (Array.isArray(values))
            return values[0];
        return Object.values(values)[0];
    }
    return values;
}
function encodeData(...parameters) {
    const [abiFunction, args = []] = (() => {
        if (Array.isArray(parameters[0])) {
            const [abi, name, args] = parameters;
            return [fromAbi(abi, name, { args }), args];
        }
        const [abiFunction, args] = parameters;
        return [abiFunction, args];
    })();
    const { overloads } = abiFunction;
    const item = overloads
        ? fromAbi([abiFunction, ...overloads], abiFunction.name, {
            args,
        })
        : abiFunction;
    const selector = getSelector(item);
    const data = args.length > 0 ? AbiParameters.encode(item.inputs, args) : undefined;
    return data ? Hex.concat(selector, data) : selector;
}
function encodeResult(...parameters) {
    const [abiFunction, output, options = {}] = (() => {
        if (Array.isArray(parameters[0])) {
            const [abi, name, output, options] = parameters;
            return [fromAbi(abi, name), output, options];
        }
        return parameters;
    })();
    const { as = 'Array' } = options;
    const values = (() => {
        if (abiFunction.outputs.length === 1)
            return [output];
        if (Array.isArray(output))
            return output;
        if (as === 'Object')
            return Object.values(output);
        return [output];
    })();
    return AbiParameters.encode(abiFunction.outputs, values);
}
function format(abiFunction) {
    return abitype.formatAbiItem(abiFunction);
}
function from(abiFunction, options = {}) {
    return AbiItem.from(abiFunction, options);
}
function fromAbi(abi, name, options) {
    const item = AbiItem.fromAbi(abi, name, options);
    if (item.type !== 'function')
        throw new AbiItem.NotFoundError({ name, type: 'function' });
    return item;
}
function getSelector(abiItem) {
    return AbiItem.getSelector(abiItem);
}
//# sourceMappingURL=AbiFunction.js.map