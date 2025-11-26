import * as abitype from 'abitype';
import * as AbiItem from './AbiItem.js';
import * as AbiParameters from './AbiParameters.js';
import * as Hex from './Hex.js';
// eslint-disable-next-line jsdoc/require-jsdoc
export function decodeData(...parameters) {
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
// eslint-disable-next-line jsdoc/require-jsdoc
export function decodeResult(...parameters) {
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
// eslint-disable-next-line jsdoc/require-jsdoc
export function encodeData(...parameters) {
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
// eslint-disable-next-line jsdoc/require-jsdoc
export function encodeResult(...parameters) {
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
/**
 * Formats an {@link ox#AbiFunction.AbiFunction} into a **Human Readable ABI Function**.
 *
 * @example
 * ```ts twoslash
 * import { AbiFunction } from 'ox'
 *
 * const formatted = AbiFunction.format({
 *   type: 'function',
 *   name: 'approve',
 *   stateMutability: 'nonpayable',
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
 *   outputs: [{ type: 'bool' }],
 * })
 *
 * formatted
 * //    ^?
 *
 *
 * ```
 *
 * @param abiFunction - The ABI Function to format.
 * @returns The formatted ABI Function.
 */
export function format(abiFunction) {
    return abitype.formatAbiItem(abiFunction);
}
/**
 * Parses an arbitrary **JSON ABI Function** or **Human Readable ABI Function** into a typed {@link ox#AbiFunction.AbiFunction}.
 *
 * @example
 * ### JSON ABIs
 *
 * ```ts twoslash
 * import { AbiFunction } from 'ox'
 *
 * const approve = AbiFunction.from({
 *   type: 'function',
 *   name: 'approve',
 *   stateMutability: 'nonpayable',
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
 *   outputs: [{ type: 'bool' }],
 * })
 *
 * approve
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
 * import { AbiFunction } from 'ox'
 *
 * const approve = AbiFunction.from(
 *   'function approve(address spender, uint256 amount) returns (bool)' // [!code hl]
 * )
 *
 * approve
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
 * import { AbiFunction } from 'ox'
 *
 * const approve = AbiFunction.from([
 *   'struct Foo { address spender; uint256 amount; }', // [!code hl]
 *   'function approve(Foo foo) returns (bool)',
 * ])
 *
 * approve
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
 * @param abiFunction - The ABI Function to parse.
 * @returns Typed ABI Function.
 */
export function from(abiFunction, options = {}) {
    return AbiItem.from(abiFunction, options);
}
/**
 * Extracts an {@link ox#AbiFunction.AbiFunction} from an {@link ox#Abi.Abi} given a name and optional arguments.
 *
 * @example
 * ### Extracting by Name
 *
 * ABI Functions can be extracted by their name using the `name` option:
 *
 * ```ts twoslash
 * import { Abi, AbiFunction } from 'ox'
 *
 * const abi = Abi.from([
 *   'function foo()',
 *   'event Transfer(address owner, address to, uint256 tokenId)',
 *   'function bar(string a) returns (uint256 x)',
 * ])
 *
 * const item = AbiFunction.fromAbi(abi, 'foo') // [!code focus]
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
 * ABI Functions can be extract by their selector when {@link ox#Hex.Hex} is provided to `name`.
 *
 * ```ts twoslash
 * import { Abi, AbiFunction } from 'ox'
 *
 * const abi = Abi.from([
 *   'function foo()',
 *   'event Transfer(address owner, address to, uint256 tokenId)',
 *   'function bar(string a) returns (uint256 x)',
 * ])
 * const item = AbiFunction.fromAbi(abi, '0x095ea7b3') // [!code focus]
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
 * Extracting via a hex selector is useful when extracting an ABI Function from an `eth_call` RPC response or
 * from a Transaction `input`.
 *
 * :::
 *
 * @param abi - The ABI to extract from.
 * @param name - The name (or selector) of the ABI item to extract.
 * @param options - Extraction options.
 * @returns The ABI item.
 */
export function fromAbi(abi, name, options) {
    const item = AbiItem.fromAbi(abi, name, options);
    if (item.type !== 'function')
        throw new AbiItem.NotFoundError({ name, type: 'function' });
    return item;
}
/**
 * Computes the [4-byte selector](https://solidity-by-example.org/function-selector/) for an {@link ox#AbiFunction.AbiFunction}.
 *
 * Useful for computing function selectors for calldata.
 *
 * @example
 * ```ts twoslash
 * import { AbiFunction } from 'ox'
 *
 * const selector = AbiFunction.getSelector('function ownerOf(uint256 tokenId)')
 * // @log: '0x6352211e'
 * ```
 *
 * @example
 * ```ts twoslash
 * import { AbiFunction } from 'ox'
 *
 * const selector = AbiFunction.getSelector({
 *   inputs: [{ type: 'uint256' }],
 *   name: 'ownerOf',
 *   outputs: [],
 *   stateMutability: 'view',
 *   type: 'function'
 * })
 * // @log: '0x6352211e'
 * ```
 *
 * @param abiItem - The ABI item to compute the selector for.
 * @returns The first 4 bytes of the {@link ox#Hash.(keccak256:function)} hash of the function signature.
 */
export function getSelector(abiItem) {
    return AbiItem.getSelector(abiItem);
}
//# sourceMappingURL=AbiFunction.js.map