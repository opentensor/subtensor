import { u8aConcatStrict } from './concat.js';
import { u8aEq } from './eq.js';
import { u8aToU8a } from './toU8a.js';
/** @internal */
export const U8A_WRAP_ETHEREUM = /*#__PURE__*/ u8aToU8a('\x19Ethereum Signed Message:\n');
/** @internal */
export const U8A_WRAP_PREFIX = /*#__PURE__*/ u8aToU8a('<Bytes>');
/** @internal */
export const U8A_WRAP_POSTFIX = /*#__PURE__*/ u8aToU8a('</Bytes>');
const WRAP_LEN = U8A_WRAP_PREFIX.length + U8A_WRAP_POSTFIX.length;
/** @internal */
export function u8aIsWrapped(u8a, withEthereum) {
    return ((u8a.length >= WRAP_LEN &&
        u8aEq(u8a.subarray(0, U8A_WRAP_PREFIX.length), U8A_WRAP_PREFIX) &&
        u8aEq(u8a.slice(-U8A_WRAP_POSTFIX.length), U8A_WRAP_POSTFIX)) ||
        (withEthereum &&
            u8a.length >= U8A_WRAP_ETHEREUM.length &&
            u8aEq(u8a.subarray(0, U8A_WRAP_ETHEREUM.length), U8A_WRAP_ETHEREUM)));
}
/**
 * @name u8aUnwrapBytes
 * @description Removes all <Bytes>...</Bytes> wrappers from the supplied value
 */
export function u8aUnwrapBytes(bytes) {
    const u8a = u8aToU8a(bytes);
    // we don't want to unwrap Ethereum-style wraps
    return u8aIsWrapped(u8a, false)
        ? u8a.subarray(U8A_WRAP_PREFIX.length, u8a.length - U8A_WRAP_POSTFIX.length)
        : u8a;
}
/**
 * @name u8aWrapBytes
 * @description
 * Adds a <Bytes>...</Bytes> wrapper to the supplied value, if
 * - We don't already have a Bytes wrapper
 * - The message is not an Ethereum-style message
 */
export function u8aWrapBytes(bytes) {
    const u8a = u8aToU8a(bytes);
    return u8aIsWrapped(u8a, true)
        ? u8a
        : u8aConcatStrict([U8A_WRAP_PREFIX, u8a, U8A_WRAP_POSTFIX]);
}
