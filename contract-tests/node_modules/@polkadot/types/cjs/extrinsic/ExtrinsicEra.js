"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GenericExtrinsicEra = exports.MortalEra = exports.ImmortalEra = void 0;
const types_codec_1 = require("@polkadot/types-codec");
const util_1 = require("@polkadot/util");
const constants_js_1 = require("./constants.js");
function getTrailingZeros(period) {
    const binary = period.toString(2);
    let index = 0;
    while (binary[binary.length - 1 - index] === '0') {
        index++;
    }
    return index;
}
/** @internal */
function decodeMortalEra(registry, value) {
    if ((0, util_1.isU8a)(value) || (0, util_1.isHex)(value) || Array.isArray(value)) {
        return decodeMortalU8a(registry, (0, util_1.u8aToU8a)(value));
    }
    else if (!value) {
        return [new types_codec_1.U64(registry), new types_codec_1.U64(registry)];
    }
    else if ((0, util_1.isObject)(value)) {
        return decodeMortalObject(registry, value);
    }
    throw new Error('Invalid data passed to Mortal era');
}
/** @internal */
function decodeMortalObject(registry, value) {
    const { current, period } = value;
    let calPeriod = Math.pow(2, Math.ceil(Math.log2(period)));
    calPeriod = Math.min(Math.max(calPeriod, 4), 1 << 16);
    const phase = current % calPeriod;
    const quantizeFactor = Math.max(calPeriod >> 12, 1);
    const quantizedPhase = phase / quantizeFactor * quantizeFactor;
    return [new types_codec_1.U64(registry, calPeriod), new types_codec_1.U64(registry, quantizedPhase)];
}
/** @internal */
function decodeMortalU8a(registry, value) {
    if (value.length === 0) {
        return [new types_codec_1.U64(registry), new types_codec_1.U64(registry)];
    }
    const first = (0, util_1.u8aToBn)(value.subarray(0, 1)).toNumber();
    const second = (0, util_1.u8aToBn)(value.subarray(1, 2)).toNumber();
    const encoded = first + (second << 8);
    const period = 2 << (encoded % (1 << 4));
    const quantizeFactor = Math.max(period >> 12, 1);
    const phase = (encoded >> 4) * quantizeFactor;
    if (period < 4 || phase >= period) {
        throw new Error('Invalid data passed to Mortal era');
    }
    return [new types_codec_1.U64(registry, period), new types_codec_1.U64(registry, phase)];
}
/** @internal */
function decodeExtrinsicEra(value = new Uint8Array()) {
    if ((0, util_1.isU8a)(value)) {
        return (!value.length || value[0] === 0)
            ? new Uint8Array([0])
            : new Uint8Array([1, value[0], value[1]]);
    }
    else if (!value) {
        return new Uint8Array([0]);
    }
    else if (value instanceof GenericExtrinsicEra) {
        return decodeExtrinsicEra(value.toU8a());
    }
    else if ((0, util_1.isHex)(value)) {
        return decodeExtrinsicEra((0, util_1.hexToU8a)(value));
    }
    else if ((0, util_1.isObject)(value)) {
        const entries = Object.entries(value).map(([k, v]) => [k.toLowerCase(), v]);
        const mortal = entries.find(([k]) => k.toLowerCase() === 'mortalera');
        const immortal = entries.find(([k]) => k.toLowerCase() === 'immortalera');
        // this is to de-serialize from JSON
        return mortal
            ? { MortalEra: mortal[1] }
            : immortal
                ? { ImmortalEra: immortal[1] }
                : { MortalEra: value };
    }
    throw new Error('Invalid data passed to Era');
}
/**
 * @name ImmortalEra
 * @description
 * The ImmortalEra for an extrinsic
 */
class ImmortalEra extends types_codec_1.Raw {
    constructor(registry, _value) {
        // For immortals, we always provide the known value (i.e. treated as a
        // constant no matter how it is constructed - it is a fixed structure)
        super(registry, constants_js_1.IMMORTAL_ERA);
    }
}
exports.ImmortalEra = ImmortalEra;
/**
 * @name MortalEra
 * @description
 * The MortalEra for an extrinsic, indicating period and phase
 */
class MortalEra extends types_codec_1.Tuple {
    constructor(registry, value) {
        super(registry, {
            period: types_codec_1.U64,
            phase: types_codec_1.U64
        }, decodeMortalEra(registry, value));
    }
    /**
     * @description Encoded length for mortals occupy 2 bytes, different from the actual Tuple since it is encoded. This is a shortcut fro `toU8a().length`
     */
    get encodedLength() {
        return 2 | 0;
    }
    /**
     * @description The period of this Mortal wraps as a [[U64]]
     */
    get period() {
        return this[0];
    }
    /**
     * @description The phase of this Mortal wraps as a [[U64]]
     */
    get phase() {
        return this[1];
    }
    /**
     * @description Converts the Object to to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    toHuman() {
        return {
            period: (0, util_1.formatNumber)(this.period),
            phase: (0, util_1.formatNumber)(this.phase)
        };
    }
    /**
     * @description Returns a JSON representation of the actual value
     */
    toJSON() {
        return this.toHex();
    }
    /**
     * @description Encodes the value as a Uint8Array as per the parity-codec specifications
     * @param isBare true when the value has none of the type-specific prefixes (internal)
     * Period and phase are encoded:
     *   - The period of validity from the block hash found in the signing material.
     *   - The phase in the period that this transaction's lifetime begins (and, importantly,
     *     implies which block hash is included in the signature material). If the `period` is
     *     greater than 1 << 12, then it will be a factor of the times greater than 1<<12 that
     *     `period` is.
     */
    toU8a(_isBare) {
        const period = this.period.toNumber();
        const encoded = Math.min(15, Math.max(1, getTrailingZeros(period) - 1)) + ((this.phase.toNumber() / Math.max(period >> 12, 1)) << 4);
        return new Uint8Array([
            encoded & 0xff,
            encoded >> 8
        ]);
    }
    /**
     * @description Get the block number of the start of the era whose properties this object describes that `current` belongs to.
     */
    birth(current) {
        const phase = this.phase.toNumber();
        const period = this.period.toNumber();
        // FIXME No toNumber() here
        return (~~((Math.max((0, util_1.bnToBn)(current).toNumber(), phase) - phase) / period) * period) + phase;
    }
    /**
     * @description Get the block number of the first block at which the era has ended.
     */
    death(current) {
        // FIXME No toNumber() here
        return this.birth(current) + this.period.toNumber();
    }
}
exports.MortalEra = MortalEra;
/**
 * @name GenericExtrinsicEra
 * @description
 * The era for an extrinsic, indicating either a mortal or immortal extrinsic
 */
class GenericExtrinsicEra extends types_codec_1.Enum {
    constructor(registry, value) {
        super(registry, {
            ImmortalEra,
            MortalEra
        }, decodeExtrinsicEra(value));
    }
    /**
     * @description Override the encoded length method
     */
    get encodedLength() {
        return this.isImmortalEra
            ? this.asImmortalEra.encodedLength
            : this.asMortalEra.encodedLength;
    }
    /**
     * @description Returns the item as a [[ImmortalEra]]
     */
    get asImmortalEra() {
        if (!this.isImmortalEra) {
            throw new Error(`Cannot convert '${this.type}' via asImmortalEra`);
        }
        return this.inner;
    }
    /**
     * @description Returns the item as a [[MortalEra]]
     */
    get asMortalEra() {
        if (!this.isMortalEra) {
            throw new Error(`Cannot convert '${this.type}' via asMortalEra`);
        }
        return this.inner;
    }
    /**
     * @description `true` if Immortal
     */
    get isImmortalEra() {
        return this.index === 0;
    }
    /**
     * @description `true` if Mortal
     */
    get isMortalEra() {
        return this.index > 0;
    }
    /**
     * @description Encodes the value as a Uint8Array as per the parity-codec specifications
     * @param isBare true when the value has none of the type-specific prefixes (internal)
     */
    toU8a(isBare) {
        return this.isMortalEra
            ? this.asMortalEra.toU8a(isBare)
            : this.asImmortalEra.toU8a(isBare);
    }
}
exports.GenericExtrinsicEra = GenericExtrinsicEra;
