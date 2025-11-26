"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.parseSignature = parseSignature;
exports.parseFunctionSignature = parseFunctionSignature;
exports.parseEventSignature = parseEventSignature;
exports.parseErrorSignature = parseErrorSignature;
exports.parseConstructorSignature = parseConstructorSignature;
exports.parseFallbackSignature = parseFallbackSignature;
exports.parseAbiParameter = parseAbiParameter;
exports.splitParameters = splitParameters;
exports.isSolidityType = isSolidityType;
exports.isSolidityKeyword = isSolidityKeyword;
exports.isValidDataLocation = isValidDataLocation;
const regex_js_1 = require("../../regex.js");
const abiItem_js_1 = require("../errors/abiItem.js");
const abiParameter_js_1 = require("../errors/abiParameter.js");
const signature_js_1 = require("../errors/signature.js");
const splitParameters_js_1 = require("../errors/splitParameters.js");
const cache_js_1 = require("./cache.js");
const signatures_js_1 = require("./signatures.js");
function parseSignature(signature, structs = {}) {
    if ((0, signatures_js_1.isFunctionSignature)(signature))
        return parseFunctionSignature(signature, structs);
    if ((0, signatures_js_1.isEventSignature)(signature))
        return parseEventSignature(signature, structs);
    if ((0, signatures_js_1.isErrorSignature)(signature))
        return parseErrorSignature(signature, structs);
    if ((0, signatures_js_1.isConstructorSignature)(signature))
        return parseConstructorSignature(signature, structs);
    if ((0, signatures_js_1.isFallbackSignature)(signature))
        return parseFallbackSignature(signature);
    if ((0, signatures_js_1.isReceiveSignature)(signature))
        return {
            type: 'receive',
            stateMutability: 'payable',
        };
    throw new signature_js_1.UnknownSignatureError({ signature });
}
function parseFunctionSignature(signature, structs = {}) {
    const match = (0, signatures_js_1.execFunctionSignature)(signature);
    if (!match)
        throw new signature_js_1.InvalidSignatureError({ signature, type: 'function' });
    const inputParams = splitParameters(match.parameters);
    const inputs = [];
    const inputLength = inputParams.length;
    for (let i = 0; i < inputLength; i++) {
        inputs.push(parseAbiParameter(inputParams[i], {
            modifiers: signatures_js_1.functionModifiers,
            structs,
            type: 'function',
        }));
    }
    const outputs = [];
    if (match.returns) {
        const outputParams = splitParameters(match.returns);
        const outputLength = outputParams.length;
        for (let i = 0; i < outputLength; i++) {
            outputs.push(parseAbiParameter(outputParams[i], {
                modifiers: signatures_js_1.functionModifiers,
                structs,
                type: 'function',
            }));
        }
    }
    return {
        name: match.name,
        type: 'function',
        stateMutability: match.stateMutability ?? 'nonpayable',
        inputs,
        outputs,
    };
}
function parseEventSignature(signature, structs = {}) {
    const match = (0, signatures_js_1.execEventSignature)(signature);
    if (!match)
        throw new signature_js_1.InvalidSignatureError({ signature, type: 'event' });
    const params = splitParameters(match.parameters);
    const abiParameters = [];
    const length = params.length;
    for (let i = 0; i < length; i++)
        abiParameters.push(parseAbiParameter(params[i], {
            modifiers: signatures_js_1.eventModifiers,
            structs,
            type: 'event',
        }));
    return { name: match.name, type: 'event', inputs: abiParameters };
}
function parseErrorSignature(signature, structs = {}) {
    const match = (0, signatures_js_1.execErrorSignature)(signature);
    if (!match)
        throw new signature_js_1.InvalidSignatureError({ signature, type: 'error' });
    const params = splitParameters(match.parameters);
    const abiParameters = [];
    const length = params.length;
    for (let i = 0; i < length; i++)
        abiParameters.push(parseAbiParameter(params[i], { structs, type: 'error' }));
    return { name: match.name, type: 'error', inputs: abiParameters };
}
function parseConstructorSignature(signature, structs = {}) {
    const match = (0, signatures_js_1.execConstructorSignature)(signature);
    if (!match)
        throw new signature_js_1.InvalidSignatureError({ signature, type: 'constructor' });
    const params = splitParameters(match.parameters);
    const abiParameters = [];
    const length = params.length;
    for (let i = 0; i < length; i++)
        abiParameters.push(parseAbiParameter(params[i], { structs, type: 'constructor' }));
    return {
        type: 'constructor',
        stateMutability: match.stateMutability ?? 'nonpayable',
        inputs: abiParameters,
    };
}
function parseFallbackSignature(signature) {
    const match = (0, signatures_js_1.execFallbackSignature)(signature);
    if (!match)
        throw new signature_js_1.InvalidSignatureError({ signature, type: 'fallback' });
    return {
        type: 'fallback',
        stateMutability: match.stateMutability ?? 'nonpayable',
    };
}
const abiParameterWithoutTupleRegex = /^(?<type>[a-zA-Z$_][a-zA-Z0-9$_]*)(?<array>(?:\[\d*?\])+?)?(?:\s(?<modifier>calldata|indexed|memory|storage{1}))?(?:\s(?<name>[a-zA-Z$_][a-zA-Z0-9$_]*))?$/;
const abiParameterWithTupleRegex = /^\((?<type>.+?)\)(?<array>(?:\[\d*?\])+?)?(?:\s(?<modifier>calldata|indexed|memory|storage{1}))?(?:\s(?<name>[a-zA-Z$_][a-zA-Z0-9$_]*))?$/;
const dynamicIntegerRegex = /^u?int$/;
function parseAbiParameter(param, options) {
    const parameterCacheKey = (0, cache_js_1.getParameterCacheKey)(param, options?.type, options?.structs);
    if (cache_js_1.parameterCache.has(parameterCacheKey))
        return cache_js_1.parameterCache.get(parameterCacheKey);
    const isTuple = regex_js_1.isTupleRegex.test(param);
    const match = (0, regex_js_1.execTyped)(isTuple ? abiParameterWithTupleRegex : abiParameterWithoutTupleRegex, param);
    if (!match)
        throw new abiParameter_js_1.InvalidParameterError({ param });
    if (match.name && isSolidityKeyword(match.name))
        throw new abiParameter_js_1.SolidityProtectedKeywordError({ param, name: match.name });
    const name = match.name ? { name: match.name } : {};
    const indexed = match.modifier === 'indexed' ? { indexed: true } : {};
    const structs = options?.structs ?? {};
    let type;
    let components = {};
    if (isTuple) {
        type = 'tuple';
        const params = splitParameters(match.type);
        const components_ = [];
        const length = params.length;
        for (let i = 0; i < length; i++) {
            components_.push(parseAbiParameter(params[i], { structs }));
        }
        components = { components: components_ };
    }
    else if (match.type in structs) {
        type = 'tuple';
        components = { components: structs[match.type] };
    }
    else if (dynamicIntegerRegex.test(match.type)) {
        type = `${match.type}256`;
    }
    else {
        type = match.type;
        if (!(options?.type === 'struct') && !isSolidityType(type))
            throw new abiItem_js_1.UnknownSolidityTypeError({ type });
    }
    if (match.modifier) {
        if (!options?.modifiers?.has?.(match.modifier))
            throw new abiParameter_js_1.InvalidModifierError({
                param,
                type: options?.type,
                modifier: match.modifier,
            });
        if (signatures_js_1.functionModifiers.has(match.modifier) &&
            !isValidDataLocation(type, !!match.array))
            throw new abiParameter_js_1.InvalidFunctionModifierError({
                param,
                type: options?.type,
                modifier: match.modifier,
            });
    }
    const abiParameter = {
        type: `${type}${match.array ?? ''}`,
        ...name,
        ...indexed,
        ...components,
    };
    cache_js_1.parameterCache.set(parameterCacheKey, abiParameter);
    return abiParameter;
}
function splitParameters(params, result = [], current = '', depth = 0) {
    const length = params.trim().length;
    for (let i = 0; i < length; i++) {
        const char = params[i];
        const tail = params.slice(i + 1);
        switch (char) {
            case ',':
                return depth === 0
                    ? splitParameters(tail, [...result, current.trim()])
                    : splitParameters(tail, result, `${current}${char}`, depth);
            case '(':
                return splitParameters(tail, result, `${current}${char}`, depth + 1);
            case ')':
                return splitParameters(tail, result, `${current}${char}`, depth - 1);
            default:
                return splitParameters(tail, result, `${current}${char}`, depth);
        }
    }
    if (current === '')
        return result;
    if (depth !== 0)
        throw new splitParameters_js_1.InvalidParenthesisError({ current, depth });
    result.push(current.trim());
    return result;
}
function isSolidityType(type) {
    return (type === 'address' ||
        type === 'bool' ||
        type === 'function' ||
        type === 'string' ||
        regex_js_1.bytesRegex.test(type) ||
        regex_js_1.integerRegex.test(type));
}
const protectedKeywordsRegex = /^(?:after|alias|anonymous|apply|auto|byte|calldata|case|catch|constant|copyof|default|defined|error|event|external|false|final|function|immutable|implements|in|indexed|inline|internal|let|mapping|match|memory|mutable|null|of|override|partial|private|promise|public|pure|reference|relocatable|return|returns|sizeof|static|storage|struct|super|supports|switch|this|true|try|typedef|typeof|var|view|virtual)$/;
function isSolidityKeyword(name) {
    return (name === 'address' ||
        name === 'bool' ||
        name === 'function' ||
        name === 'string' ||
        name === 'tuple' ||
        regex_js_1.bytesRegex.test(name) ||
        regex_js_1.integerRegex.test(name) ||
        protectedKeywordsRegex.test(name));
}
function isValidDataLocation(type, isArray) {
    return isArray || type === 'bytes' || type === 'string' || type === 'tuple';
}
//# sourceMappingURL=utils.js.map