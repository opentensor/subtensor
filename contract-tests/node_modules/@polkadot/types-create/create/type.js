import { Option } from '@polkadot/types-codec';
import { isHex, isU8a, u8aEq, u8aToHex, u8aToU8a } from '@polkadot/util';
import { createClassUnsafe } from './class.js';
function checkInstance(created, matcher) {
    const u8a = created.toU8a();
    const rawType = created.toRawType();
    const isOk = (
    // full match, all ok
    u8aEq(u8a, matcher) ||
        (
        // on a length-prefixed type, just check the actual length
        ['Bytes', 'Text', 'Type'].includes(rawType) &&
            matcher.length === created.length) ||
        (
        // when the created is empty and matcher is also empty, let it slide...
        created.isEmpty &&
            matcher.every((v) => !v)));
    if (!isOk) {
        throw new Error(`${rawType}:: Decoded input doesn't match input, received ${u8aToHex(matcher, 512)} (${matcher.length} bytes), created ${u8aToHex(u8a, 512)} (${u8a.length} bytes)`);
    }
}
function checkPedantic(created, [value]) {
    if (isU8a(value)) {
        checkInstance(created, value);
    }
    else if (isHex(value)) {
        checkInstance(created, u8aToU8a(value));
    }
}
function initType(registry, Type, params = [], { blockHash, isFallback, isOptional, isPedantic } = {}) {
    const created = new (isOptional
        ? Option.with(Type)
        : Type)(registry, ...params);
    isPedantic && checkPedantic(created, params);
    if (blockHash) {
        created.createdAtHash = createTypeUnsafe(registry, 'BlockHash', [blockHash]);
    }
    if (isFallback) {
        created.isStorageFallback = true;
    }
    return created;
}
export function createTypeUnsafe(registry, type, params = [], options = {}) {
    let Clazz = null;
    let firstError = null;
    try {
        Clazz = createClassUnsafe(registry, type);
        return initType(registry, Clazz, params, options);
    }
    catch (error) {
        firstError = new Error(`createType(${type}):: ${error.message}`);
    }
    if (Clazz?.__fallbackType) {
        try {
            Clazz = createClassUnsafe(registry, Clazz.__fallbackType);
            return initType(registry, Clazz, params, options);
        }
        catch {
            // swallow, we will throw the first error again
        }
    }
    throw firstError;
}
