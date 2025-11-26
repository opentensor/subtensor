import { bytesRegex, execTyped, integerRegex, isTupleRegex, } from '../../regex.js';
import { UnknownSolidityTypeError } from '../errors/abiItem.js';
import { InvalidFunctionModifierError, InvalidModifierError, InvalidParameterError, SolidityProtectedKeywordError, } from '../errors/abiParameter.js';
import { InvalidSignatureError, UnknownSignatureError, } from '../errors/signature.js';
import { InvalidParenthesisError } from '../errors/splitParameters.js';
import { getParameterCacheKey, parameterCache } from './cache.js';
import { eventModifiers, execConstructorSignature, execErrorSignature, execEventSignature, execFallbackSignature, execFunctionSignature, functionModifiers, isConstructorSignature, isErrorSignature, isEventSignature, isFallbackSignature, isFunctionSignature, isReceiveSignature, } from './signatures.js';
export function parseSignature(signature, structs = {}) {
    if (isFunctionSignature(signature))
        return parseFunctionSignature(signature, structs);
    if (isEventSignature(signature))
        return parseEventSignature(signature, structs);
    if (isErrorSignature(signature))
        return parseErrorSignature(signature, structs);
    if (isConstructorSignature(signature))
        return parseConstructorSignature(signature, structs);
    if (isFallbackSignature(signature))
        return parseFallbackSignature(signature);
    if (isReceiveSignature(signature))
        return {
            type: 'receive',
            stateMutability: 'payable',
        };
    throw new UnknownSignatureError({ signature });
}
export function parseFunctionSignature(signature, structs = {}) {
    const match = execFunctionSignature(signature);
    if (!match)
        throw new InvalidSignatureError({ signature, type: 'function' });
    const inputParams = splitParameters(match.parameters);
    const inputs = [];
    const inputLength = inputParams.length;
    for (let i = 0; i < inputLength; i++) {
        inputs.push(parseAbiParameter(inputParams[i], {
            modifiers: functionModifiers,
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
                modifiers: functionModifiers,
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
export function parseEventSignature(signature, structs = {}) {
    const match = execEventSignature(signature);
    if (!match)
        throw new InvalidSignatureError({ signature, type: 'event' });
    const params = splitParameters(match.parameters);
    const abiParameters = [];
    const length = params.length;
    for (let i = 0; i < length; i++)
        abiParameters.push(parseAbiParameter(params[i], {
            modifiers: eventModifiers,
            structs,
            type: 'event',
        }));
    return { name: match.name, type: 'event', inputs: abiParameters };
}
export function parseErrorSignature(signature, structs = {}) {
    const match = execErrorSignature(signature);
    if (!match)
        throw new InvalidSignatureError({ signature, type: 'error' });
    const params = splitParameters(match.parameters);
    const abiParameters = [];
    const length = params.length;
    for (let i = 0; i < length; i++)
        abiParameters.push(parseAbiParameter(params[i], { structs, type: 'error' }));
    return { name: match.name, type: 'error', inputs: abiParameters };
}
export function parseConstructorSignature(signature, structs = {}) {
    const match = execConstructorSignature(signature);
    if (!match)
        throw new InvalidSignatureError({ signature, type: 'constructor' });
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
export function parseFallbackSignature(signature) {
    const match = execFallbackSignature(signature);
    if (!match)
        throw new InvalidSignatureError({ signature, type: 'fallback' });
    return {
        type: 'fallback',
        stateMutability: match.stateMutability ?? 'nonpayable',
    };
}
const abiParameterWithoutTupleRegex = /^(?<type>[a-zA-Z$_][a-zA-Z0-9$_]*)(?<array>(?:\[\d*?\])+?)?(?:\s(?<modifier>calldata|indexed|memory|storage{1}))?(?:\s(?<name>[a-zA-Z$_][a-zA-Z0-9$_]*))?$/;
const abiParameterWithTupleRegex = /^\((?<type>.+?)\)(?<array>(?:\[\d*?\])+?)?(?:\s(?<modifier>calldata|indexed|memory|storage{1}))?(?:\s(?<name>[a-zA-Z$_][a-zA-Z0-9$_]*))?$/;
const dynamicIntegerRegex = /^u?int$/;
export function parseAbiParameter(param, options) {
    // optional namespace cache by `type`
    const parameterCacheKey = getParameterCacheKey(param, options?.type, options?.structs);
    if (parameterCache.has(parameterCacheKey))
        return parameterCache.get(parameterCacheKey);
    const isTuple = isTupleRegex.test(param);
    const match = execTyped(isTuple ? abiParameterWithTupleRegex : abiParameterWithoutTupleRegex, param);
    if (!match)
        throw new InvalidParameterError({ param });
    if (match.name && isSolidityKeyword(match.name))
        throw new SolidityProtectedKeywordError({ param, name: match.name });
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
            // remove `modifiers` from `options` to prevent from being added to tuple components
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
            throw new UnknownSolidityTypeError({ type });
    }
    if (match.modifier) {
        // Check if modifier exists, but is not allowed (e.g. `indexed` in `functionModifiers`)
        if (!options?.modifiers?.has?.(match.modifier))
            throw new InvalidModifierError({
                param,
                type: options?.type,
                modifier: match.modifier,
            });
        // Check if resolved `type` is valid if there is a function modifier
        if (functionModifiers.has(match.modifier) &&
            !isValidDataLocation(type, !!match.array))
            throw new InvalidFunctionModifierError({
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
    parameterCache.set(parameterCacheKey, abiParameter);
    return abiParameter;
}
// s/o latika for this
export function splitParameters(params, result = [], current = '', depth = 0) {
    const length = params.trim().length;
    // biome-ignore lint/correctness/noUnreachable: recursive
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
        throw new InvalidParenthesisError({ current, depth });
    result.push(current.trim());
    return result;
}
export function isSolidityType(type) {
    return (type === 'address' ||
        type === 'bool' ||
        type === 'function' ||
        type === 'string' ||
        bytesRegex.test(type) ||
        integerRegex.test(type));
}
const protectedKeywordsRegex = /^(?:after|alias|anonymous|apply|auto|byte|calldata|case|catch|constant|copyof|default|defined|error|event|external|false|final|function|immutable|implements|in|indexed|inline|internal|let|mapping|match|memory|mutable|null|of|override|partial|private|promise|public|pure|reference|relocatable|return|returns|sizeof|static|storage|struct|super|supports|switch|this|true|try|typedef|typeof|var|view|virtual)$/;
/** @internal */
export function isSolidityKeyword(name) {
    return (name === 'address' ||
        name === 'bool' ||
        name === 'function' ||
        name === 'string' ||
        name === 'tuple' ||
        bytesRegex.test(name) ||
        integerRegex.test(name) ||
        protectedKeywordsRegex.test(name));
}
/** @internal */
export function isValidDataLocation(type, isArray) {
    return isArray || type === 'bytes' || type === 'string' || type === 'tuple';
}
//# sourceMappingURL=utils.js.map