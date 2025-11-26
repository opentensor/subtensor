"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sortAsc = sortAsc;
exports.sortSet = sortSet;
exports.sortMap = sortMap;
const util_1 = require("@polkadot/util");
/** @internal **/
function isArrayLike(arg) {
    return arg instanceof Uint8Array || Array.isArray(arg);
}
/** @internal **/
function isEnum(arg) {
    return (0, util_1.isCodec)(arg) && (0, util_1.isNumber)(arg.index) && (0, util_1.isCodec)(arg.value);
}
/** @internal **/
function isOption(arg) {
    return (0, util_1.isCodec)(arg) && (0, util_1.isBoolean)(arg.isSome) && (0, util_1.isCodec)(arg.value);
}
/** @internal */
function isNumberLike(arg) {
    return (0, util_1.isNumber)(arg) || (0, util_1.isBn)(arg) || (0, util_1.isBigInt)(arg);
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
    if ((0, util_1.isCodec)(arg)) {
        const hex = arg.toHex();
        // Check if we have seen the value.
        if (seen.has(hex)) {
            // Duplicates are not allowed.
            throw new Error(`Duplicate value in ${container}: ${(0, util_1.stringify)(arg)}`);
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
function sortAsc(a, b) {
    if (isNumberLike(a) && isNumberLike(b)) {
        return (0, util_1.bnToBn)(a).cmp((0, util_1.bnToBn)(b));
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
    else if ((0, util_1.isCodec)(a) && (0, util_1.isCodec)(b)) {
        // Text, Bool etc.
        return sortAsc(a.toU8a(true), b.toU8a(true));
    }
    throw new Error(`Attempting to sort unrecognized values: ${(0, util_1.stringify)(a)} (typeof ${typeof a}) <-> ${(0, util_1.stringify)(b)} (typeof ${typeof b})`);
}
function sortSet(set) {
    const seen = new Set();
    return new Set(Array.from(set).filter((value) => checkForDuplicates('BTreeSet', seen, value)).sort(sortAsc));
}
function sortMap(map) {
    const seen = new Set();
    return new Map(Array.from(map.entries()).filter(([key]) => checkForDuplicates('BTreeMap', seen, key)).sort(([keyA], [keyB]) => sortAsc(keyA, keyB)));
}
