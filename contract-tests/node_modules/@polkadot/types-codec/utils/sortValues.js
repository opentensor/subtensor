import { bnToBn, isBigInt, isBn, isBoolean, isCodec, isNumber, stringify } from '@polkadot/util';
/** @internal **/
function isArrayLike(arg) {
    return arg instanceof Uint8Array || Array.isArray(arg);
}
/** @internal **/
function isEnum(arg) {
    return isCodec(arg) && isNumber(arg.index) && isCodec(arg.value);
}
/** @internal **/
function isOption(arg) {
    return isCodec(arg) && isBoolean(arg.isSome) && isCodec(arg.value);
}
/** @internal */
function isNumberLike(arg) {
    return isNumber(arg) || isBn(arg) || isBigInt(arg);
}
/** @internal */
function sortArray(a, b) {
    // Vec, Tuple, Bytes etc.
    let sortRes = 0;
    const minLen = Math.min(a.length, b.length);
    for (let i = 0; i < minLen; ++i) {
        sortRes = sortAsc(a[i], b[i]);
        if (sortRes !== 0) {
            return sortRes;
        }
    }
    return a.length - b.length;
}
/** @internal */
function checkForDuplicates(container, seen, arg) {
    // Convert the value to hex.
    if (isCodec(arg)) {
        const hex = arg.toHex();
        // Check if we have seen the value.
        if (seen.has(hex)) {
            // Duplicates are not allowed.
            throw new Error(`Duplicate value in ${container}: ${stringify(arg)}`);
        }
        seen.add(hex);
    }
    return true;
}
/**
* Sort keys/values of BTreeSet/BTreeMap in ascending order for encoding compatibility with Rust's BTreeSet/BTreeMap
* (https://doc.rust-lang.org/stable/std/collections/struct.BTreeSet.html)
* (https://doc.rust-lang.org/stable/std/collections/struct.BTreeMap.html)
*/
export function sortAsc(a, b) {
    if (isNumberLike(a) && isNumberLike(b)) {
        return bnToBn(a).cmp(bnToBn(b));
    }
    else if (a instanceof Map && b instanceof Map) {
        return sortAsc(Array.from(a.values()), Array.from(b.values()));
    }
    else if (isEnum(a) && isEnum(b)) {
        return sortAsc(a.index, b.index) || sortAsc(a.value, b.value);
    }
    else if (isOption(a) && isOption(b)) {
        return sortAsc(a.isNone ? 0 : 1, b.isNone ? 0 : 1) || sortAsc(a.value, b.value);
    }
    else if (isArrayLike(a) && isArrayLike(b)) {
        return sortArray(a, b);
    }
    else if (isCodec(a) && isCodec(b)) {
        // Text, Bool etc.
        return sortAsc(a.toU8a(true), b.toU8a(true));
    }
    throw new Error(`Attempting to sort unrecognized values: ${stringify(a)} (typeof ${typeof a}) <-> ${stringify(b)} (typeof ${typeof b})`);
}
export function sortSet(set) {
    const seen = new Set();
    return new Set(Array.from(set).filter((value) => checkForDuplicates('BTreeSet', seen, value)).sort(sortAsc));
}
export function sortMap(map) {
    const seen = new Set();
    return new Map(Array.from(map.entries()).filter(([key]) => checkForDuplicates('BTreeMap', seen, key)).sort(([keyA], [keyB]) => sortAsc(keyA, keyB)));
}
