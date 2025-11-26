import { u8aToU8a } from './toU8a.js';
/**
 * @name u8aConcat
 * @summary Creates a concatenated Uint8Array from the inputs.
 * @description
 * Concatenates the input arrays into a single `UInt8Array`.
 * @example
 * <BR>
 *
 * ```javascript
 * import { { u8aConcat } from '@polkadot/util';
 *
 * u8aConcat(
 *   new Uint8Array([1, 2, 3]),
 *   new Uint8Array([4, 5, 6])
 * ); // [1, 2, 3, 4, 5, 6]
 * ```
 */
export function u8aConcat(...list) {
    const count = list.length;
    const u8as = new Array(count);
    let length = 0;
    for (let i = 0; i < count; i++) {
        u8as[i] = u8aToU8a(list[i]);
        length += u8as[i].length;
    }
    return u8aConcatStrict(u8as, length);
}
/**
 * @name u8aConcatStrict
 * @description A strict version of [[u8aConcat]], accepting only Uint8Array inputs
 */
export function u8aConcatStrict(u8as, length = 0) {
    const count = u8as.length;
    let offset = 0;
    if (!length) {
        for (let i = 0; i < count; i++) {
            length += u8as[i].length;
        }
    }
    const result = new Uint8Array(length);
    for (let i = 0; i < count; i++) {
        result.set(u8as[i], offset);
        offset += u8as[i].length;
    }
    return result;
}
