"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.SizeExceedsPaddingSizeError = exports.SliceOffsetOutOfBoundsError = exports.SizeOverflowError = exports.InvalidBytesTypeError = exports.InvalidBytesBooleanError = void 0;
exports.assert = assert;
exports.concat = concat;
exports.from = from;
exports.fromArray = fromArray;
exports.fromBoolean = fromBoolean;
exports.fromHex = fromHex;
exports.fromNumber = fromNumber;
exports.fromString = fromString;
exports.isEqual = isEqual;
exports.padLeft = padLeft;
exports.padRight = padRight;
exports.random = random;
exports.size = size;
exports.slice = slice;
exports.toBigInt = toBigInt;
exports.toBoolean = toBoolean;
exports.toHex = toHex;
exports.toNumber = toNumber;
exports.toString = toString;
exports.trimLeft = trimLeft;
exports.trimRight = trimRight;
exports.validate = validate;
const utils_1 = require("@noble/curves/abstract/utils");
const Errors = require("./Errors.js");
const Hex = require("./Hex.js");
const Json = require("./Json.js");
const internal = require("./internal/bytes.js");
const internal_hex = require("./internal/hex.js");
const decoder = new TextDecoder();
const encoder = new TextEncoder();
function assert(value) {
    if (value instanceof Uint8Array)
        return;
    if (!value)
        throw new InvalidBytesTypeError(value);
    if (typeof value !== 'object')
        throw new InvalidBytesTypeError(value);
    if (!('BYTES_PER_ELEMENT' in value))
        throw new InvalidBytesTypeError(value);
    if (value.BYTES_PER_ELEMENT !== 1 || value.constructor.name !== 'Uint8Array')
        throw new InvalidBytesTypeError(value);
}
function concat(...values) {
    let length = 0;
    for (const arr of values) {
        length += arr.length;
    }
    const result = new Uint8Array(length);
    for (let i = 0, index = 0; i < values.length; i++) {
        const arr = values[i];
        result.set(arr, index);
        index += arr.length;
    }
    return result;
}
function from(value) {
    if (value instanceof Uint8Array)
        return value;
    if (typeof value === 'string')
        return fromHex(value);
    return fromArray(value);
}
function fromArray(value) {
    return value instanceof Uint8Array ? value : new Uint8Array(value);
}
function fromBoolean(value, options = {}) {
    const { size } = options;
    const bytes = new Uint8Array(1);
    bytes[0] = Number(value);
    if (typeof size === 'number') {
        internal.assertSize(bytes, size);
        return padLeft(bytes, size);
    }
    return bytes;
}
function fromHex(value, options = {}) {
    const { size } = options;
    let hex = value;
    if (size) {
        internal_hex.assertSize(value, size);
        hex = Hex.padRight(value, size);
    }
    let hexString = hex.slice(2);
    if (hexString.length % 2)
        hexString = `0${hexString}`;
    const length = hexString.length / 2;
    const bytes = new Uint8Array(length);
    for (let index = 0, j = 0; index < length; index++) {
        const nibbleLeft = internal.charCodeToBase16(hexString.charCodeAt(j++));
        const nibbleRight = internal.charCodeToBase16(hexString.charCodeAt(j++));
        if (nibbleLeft === undefined || nibbleRight === undefined) {
            throw new Errors.BaseError(`Invalid byte sequence ("${hexString[j - 2]}${hexString[j - 1]}" in "${hexString}").`);
        }
        bytes[index] = nibbleLeft * 16 + nibbleRight;
    }
    return bytes;
}
function fromNumber(value, options) {
    const hex = Hex.fromNumber(value, options);
    return fromHex(hex);
}
function fromString(value, options = {}) {
    const { size } = options;
    const bytes = encoder.encode(value);
    if (typeof size === 'number') {
        internal.assertSize(bytes, size);
        return padRight(bytes, size);
    }
    return bytes;
}
function isEqual(bytesA, bytesB) {
    return (0, utils_1.equalBytes)(bytesA, bytesB);
}
function padLeft(value, size) {
    return internal.pad(value, { dir: 'left', size });
}
function padRight(value, size) {
    return internal.pad(value, { dir: 'right', size });
}
function random(length) {
    return crypto.getRandomValues(new Uint8Array(length));
}
function size(value) {
    return value.length;
}
function slice(value, start, end, options = {}) {
    const { strict } = options;
    internal.assertStartOffset(value, start);
    const value_ = value.slice(start, end);
    if (strict)
        internal.assertEndOffset(value_, start, end);
    return value_;
}
function toBigInt(bytes, options = {}) {
    const { size } = options;
    if (typeof size !== 'undefined')
        internal.assertSize(bytes, size);
    const hex = Hex.fromBytes(bytes, options);
    return Hex.toBigInt(hex, options);
}
function toBoolean(bytes, options = {}) {
    const { size } = options;
    let bytes_ = bytes;
    if (typeof size !== 'undefined') {
        internal.assertSize(bytes_, size);
        bytes_ = trimLeft(bytes_);
    }
    if (bytes_.length > 1 || bytes_[0] > 1)
        throw new InvalidBytesBooleanError(bytes_);
    return Boolean(bytes_[0]);
}
function toHex(value, options = {}) {
    return Hex.fromBytes(value, options);
}
function toNumber(bytes, options = {}) {
    const { size } = options;
    if (typeof size !== 'undefined')
        internal.assertSize(bytes, size);
    const hex = Hex.fromBytes(bytes, options);
    return Hex.toNumber(hex, options);
}
function toString(bytes, options = {}) {
    const { size } = options;
    let bytes_ = bytes;
    if (typeof size !== 'undefined') {
        internal.assertSize(bytes_, size);
        bytes_ = trimRight(bytes_);
    }
    return decoder.decode(bytes_);
}
function trimLeft(value) {
    return internal.trim(value, { dir: 'left' });
}
function trimRight(value) {
    return internal.trim(value, { dir: 'right' });
}
function validate(value) {
    try {
        assert(value);
        return true;
    }
    catch {
        return false;
    }
}
class InvalidBytesBooleanError extends Errors.BaseError {
    constructor(bytes) {
        super(`Bytes value \`${bytes}\` is not a valid boolean.`, {
            metaMessages: [
                'The bytes array must contain a single byte of either a `0` or `1` value.',
            ],
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'Bytes.InvalidBytesBooleanError'
        });
    }
}
exports.InvalidBytesBooleanError = InvalidBytesBooleanError;
class InvalidBytesTypeError extends Errors.BaseError {
    constructor(value) {
        super(`Value \`${typeof value === 'object' ? Json.stringify(value) : value}\` of type \`${typeof value}\` is an invalid Bytes value.`, {
            metaMessages: ['Bytes values must be of type `Bytes`.'],
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'Bytes.InvalidBytesTypeError'
        });
    }
}
exports.InvalidBytesTypeError = InvalidBytesTypeError;
class SizeOverflowError extends Errors.BaseError {
    constructor({ givenSize, maxSize }) {
        super(`Size cannot exceed \`${maxSize}\` bytes. Given size: \`${givenSize}\` bytes.`);
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'Bytes.SizeOverflowError'
        });
    }
}
exports.SizeOverflowError = SizeOverflowError;
class SliceOffsetOutOfBoundsError extends Errors.BaseError {
    constructor({ offset, position, size, }) {
        super(`Slice ${position === 'start' ? 'starting' : 'ending'} at offset \`${offset}\` is out-of-bounds (size: \`${size}\`).`);
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'Bytes.SliceOffsetOutOfBoundsError'
        });
    }
}
exports.SliceOffsetOutOfBoundsError = SliceOffsetOutOfBoundsError;
class SizeExceedsPaddingSizeError extends Errors.BaseError {
    constructor({ size, targetSize, type, }) {
        super(`${type.charAt(0).toUpperCase()}${type
            .slice(1)
            .toLowerCase()} size (\`${size}\`) exceeds padding size (\`${targetSize}\`).`);
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'Bytes.SizeExceedsPaddingSizeError'
        });
    }
}
exports.SizeExceedsPaddingSizeError = SizeExceedsPaddingSizeError;
//# sourceMappingURL=Bytes.js.map