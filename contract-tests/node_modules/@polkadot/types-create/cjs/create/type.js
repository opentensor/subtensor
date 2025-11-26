"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createTypeUnsafe = createTypeUnsafe;
const types_codec_1 = require("@polkadot/types-codec");
const util_1 = require("@polkadot/util");
const class_js_1 = require("./class.js");
function checkInstance(created, matcher) {
    const u8a = created.toU8a();
    const rawType = created.toRawType();
    const isOk = (
    // full match, all ok
    (0, util_1.u8aEq)(u8a, matcher) ||
        (
        // on a length-prefixed type, just check the actual length
        ['Bytes', 'Text', 'Type'].includes(rawType) &&
            matcher.length === created.length) ||
        (
        // when the created is empty and matcher is also empty, let it slide...
        created.isEmpty &&
            matcher.every((v) => !v)));
    if (!isOk) {
        throw new Error(`${rawType}:: Decoded input doesn't match input, received ${(0, util_1.u8aToHex)(matcher, 512)} (${matcher.length} bytes), created ${(0, util_1.u8aToHex)(u8a, 512)} (${u8a.length} bytes)`);
    }
}
function checkPedantic(created, [value]) {
    if ((0, util_1.isU8a)(value)) {
        checkInstance(created, value);
    }
    else if ((0, util_1.isHex)(value)) {
        checkInstance(created, (0, util_1.u8aToU8a)(value));
    }
}
function initType(registry, Type, params = [], { blockHash, isFallback, isOptional, isPedantic } = {}) {
    const created = new (isOptional
        ? types_codec_1.Option.with(Type)
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
function createTypeUnsafe(registry, type, params = [], options = {}) {
    let Clazz = null;
    let firstError = null;
    try {
        Clazz = (0, class_js_1.createClassUnsafe)(registry, type);
        return initType(registry, Clazz, params, options);
    }
    catch (error) {
        firstError = new Error(`createType(${type}):: ${error.message}`);
    }
    if (Clazz?.__fallbackType) {
        try {
            Clazz = (0, class_js_1.createClassUnsafe)(registry, Clazz.__fallbackType);
            return initType(registry, Clazz, params, options);
        }
        catch {
            // swallow, we will throw the first error again
        }
    }
    throw firstError;
}
