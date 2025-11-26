import * as abitype from 'abitype';
import * as AbiItem from './AbiItem.js';
import * as AbiParameters from './AbiParameters.js';
import * as Hex from './Hex.js';
// eslint-disable-next-line jsdoc/require-jsdoc
export function decode(...parameters) {
    const [abiError, data, options = {}] = (() => {
        if (Array.isArray(parameters[0])) {
            const [abi, name, data, options] = parameters;
            return [fromAbi(abi, name), data, options];
        }
        return parameters;
    })();
    if (Hex.size(data) < 4)
        throw new AbiItem.InvalidSelectorSizeError({ data });
    if (abiError.inputs.length === 0)
        return undefined;
    const values = AbiParameters.decode(abiError.inputs, Hex.slice(data, 4), options);
    if (values && Object.keys(values).length === 1) {
        if (Array.isArray(values))
            return values[0];
        return Object.values(values)[0];
    }
    return values;
}
// eslint-disable-next-line jsdoc/require-jsdoc
export function encode(...parameters) {
    const [abiError, args] = (() => {
        if (Array.isArray(parameters[0])) {
            const [abi, name, ...args] = parameters;
            return [fromAbi(abi, name), args];
        }
        const [abiError, ...args] = parameters;
        return [abiError, args];
    })();
    const selector = getSelector(abiError);
    const data = args.length > 0
        ? AbiParameters.encode(abiError.inputs, args[0])
        : undefined;
    return data ? Hex.concat(selector, data) : selector;
}
/**
 * Formats an {@link ox#AbiError.AbiError} into a **Human Readable ABI Error**.
 *
 * @example
 * ```ts twoslash
 * import { AbiError } from 'ox'
 *
 * const formatted = AbiError.format({
 *   type: 'error',
 *   name: 'Example',
 *   inputs: [
 *     {
 *       name: 'spender',
 *       type: 'address',
 *     },
 *     {
 *       name: 'amount',
 *       type: 'uint256',
 *     },
 *   ],
 * })
 *
 * formatted
 * //    ^?
 *
 *
 * ```
 *
 * @param abiError - The ABI Error to format.
 * @returns The formatted ABI Error.
 */
export function format(abiError) {
    return abitype.formatAbiItem(abiError);
}
/**
 * Parses an arbitrary **JSON ABI Error** or **Human Readable ABI Error** into a typed {@link ox#AbiError.AbiError}.
 *
 * @example
 * ### JSON ABIs
 *
 * ```ts twoslash
 * import { AbiError } from 'ox'
 *
 * const badSignatureVError = AbiError.from({
 *   inputs: [{ name: 'v', type: 'uint8' }],
 *   name: 'BadSignatureV',
 *   type: 'error',
 * })
 *
 * badSignatureVError
 * //^?
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 * ```
 *
 * @example
 * ### Human Readable ABIs
 *
 * A Human Readable ABI can be parsed into a typed ABI object:
 *
 * ```ts twoslash
 * import { AbiError } from 'ox'
 *
 * const badSignatureVError = AbiError.from(
 *   'error BadSignatureV(uint8 v)' // [!code hl]
 * )
 *
 * badSignatureVError
 * //^?
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 * ```
 *
 * @example
 * It is possible to specify `struct`s along with your definitions:
 *
 * ```ts twoslash
 * import { AbiError } from 'ox'
 *
 * const badSignatureVError = AbiError.from([
 *   'struct Signature { uint8 v; }', // [!code hl]
 *   'error BadSignatureV(Signature signature)',
 * ])
 *
 * badSignatureVError
 * //^?
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 * ```
 *
 *
 *
 * @param abiError - The ABI Error to parse.
 * @returns Typed ABI Error.
 */
export function from(abiError, options = {}) {
    return AbiItem.from(abiError, options);
}
/**
 * Extracts an {@link ox#AbiError.AbiError} from an {@link ox#Abi.Abi} given a name and optional arguments.
 *
 * @example
 * ### Extracting by Name
 *
 * ABI Errors can be extracted by their name using the `name` option:
 *
 * ```ts twoslash
 * import { Abi, AbiError } from 'ox'
 *
 * const abi = Abi.from([
 *   'function foo()',
 *   'error BadSignatureV(uint8 v)',
 *   'function bar(string a) returns (uint256 x)',
 * ])
 *
 * const item = AbiError.fromAbi(abi, 'BadSignatureV') // [!code focus]
 * //    ^?
 *
 *
 *
 *
 *
 *
 * ```
 *
 * @example
 * ### Extracting by Selector
 *
 * ABI Errors can be extract by their selector when {@link ox#Hex.Hex} is provided to `name`.
 *
 * ```ts twoslash
 * import { Abi, AbiError } from 'ox'
 *
 * const abi = Abi.from([
 *   'function foo()',
 *   'error BadSignatureV(uint8 v)',
 *   'function bar(string a) returns (uint256 x)',
 * ])
 * const item = AbiError.fromAbi(abi, '0x095ea7b3') // [!code focus]
 * //    ^?
 *
 *
 *
 *
 *
 *
 *
 *
 *
 * ```
 *
 * :::note
 *
 * Extracting via a hex selector is useful when extracting an ABI Error from JSON-RPC error data.
 *
 * :::
 *
 * @param abi - The ABI to extract from.
 * @param name - The name (or selector) of the ABI item to extract.
 * @param options - Extraction options.
 * @returns The ABI item.
 */
export function fromAbi(abi, name, options) {
    if (name === 'Error')
        return solidityError;
    if (name === 'Panic')
        return solidityPanic;
    if (Hex.validate(name, { strict: false })) {
        const selector = Hex.slice(name, 0, 4);
        if (selector === solidityErrorSelector)
            return solidityError;
        if (selector === solidityPanicSelector)
            return solidityPanic;
    }
    const item = AbiItem.fromAbi(abi, name, options);
    if (item.type !== 'error')
        throw new AbiItem.NotFoundError({ name, type: 'error' });
    return item;
}
/**
 * Computes the [4-byte selector](https://solidity-by-example.org/function-selector/) for an {@link ox#AbiError.AbiError}.
 *
 * @example
 * ```ts twoslash
 * import { AbiError } from 'ox'
 *
 * const selector = AbiError.getSelector('error BadSignatureV(uint8 v)')
 * // @log: '0x6352211e'
 * ```
 *
 * @example
 * ```ts twoslash
 * import { AbiError } from 'ox'
 *
 * const selector = AbiError.getSelector({
 *   inputs: [{ name: 'v', type: 'uint8' }],
 *   name: 'BadSignatureV',
 *   type: 'error'
 * })
 * // @log: '0x6352211e'
 * ```
 *
 * @param abiItem - The ABI item to compute the selector for.
 * @returns The first 4 bytes of the {@link ox#Hash.(keccak256:function)} hash of the error signature.
 */
export function getSelector(abiItem) {
    return AbiItem.getSelector(abiItem);
}
// https://docs.soliditylang.org/en/v0.8.16/control-structures.html#panic-via-assert-and-error-via-require
export const panicReasons = {
    1: 'An `assert` condition failed.',
    17: 'Arithmetic operation resulted in underflow or overflow.',
    18: 'Division or modulo by zero (e.g. `5 / 0` or `23 % 0`).',
    33: 'Attempted to convert to an invalid type.',
    34: 'Attempted to access a storage byte array that is incorrectly encoded.',
    49: 'Performed `.pop()` on an empty array',
    50: 'Array index is out of bounds.',
    65: 'Allocated too much memory or created an array which is too large.',
    81: 'Attempted to call a zero-initialized variable of internal function type.',
};
export const solidityError = /*#__PURE__*/ from({
    inputs: [
        {
            name: 'message',
            type: 'string',
        },
    ],
    name: 'Error',
    type: 'error',
});
export const solidityErrorSelector = '0x08c379a0';
export const solidityPanic = /*#__PURE__*/ from({
    inputs: [
        {
            name: 'reason',
            type: 'uint8',
        },
    ],
    name: 'Panic',
    type: 'error',
});
export const solidityPanicSelector = '0x4e487b71';
//# sourceMappingURL=AbiError.js.map