"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.xxhash64 = xxhash64;
const util_1 = require("@polkadot/util");
const x_bigint_1 = require("@polkadot/x-bigint");
const P64_1 = (0, x_bigint_1.BigInt)('11400714785074694791');
const P64_2 = (0, x_bigint_1.BigInt)('14029467366897019727');
const P64_3 = (0, x_bigint_1.BigInt)('1609587929392839161');
const P64_4 = (0, x_bigint_1.BigInt)('9650029242287828579');
const P64_5 = (0, x_bigint_1.BigInt)('2870177450012600261');
const U64 = (0, x_bigint_1.BigInt)('0xffffffffffffffff');
const _7n = (0, x_bigint_1.BigInt)(7);
const _11n = (0, x_bigint_1.BigInt)(11);
const _12n = (0, x_bigint_1.BigInt)(12);
const _16n = (0, x_bigint_1.BigInt)(16);
const _18n = (0, x_bigint_1.BigInt)(18);
const _23n = (0, x_bigint_1.BigInt)(23);
const _27n = (0, x_bigint_1.BigInt)(27);
const _29n = (0, x_bigint_1.BigInt)(29);
const _31n = (0, x_bigint_1.BigInt)(31);
const _32n = (0, x_bigint_1.BigInt)(32);
const _33n = (0, x_bigint_1.BigInt)(33);
const _64n = (0, x_bigint_1.BigInt)(64);
const _256n = (0, x_bigint_1.BigInt)(256);
function rotl(a, b) {
    const c = a & U64;
    return ((c << b) | (c >> (_64n - b))) & U64;
}
function fromU8a(u8a, p, count) {
    const bigints = new Array(count);
    let offset = 0;
    for (let i = 0; i < count; i++, offset += 2) {
        bigints[i] = (0, x_bigint_1.BigInt)(u8a[p + offset] | (u8a[p + 1 + offset] << 8));
    }
    let result = util_1._0n;
    for (let i = count - 1; i >= 0; i--) {
        result = (result << _16n) + bigints[i];
    }
    return result;
}
function init(seed, input) {
    const state = {
        seed,
        u8a: new Uint8Array(32),
        u8asize: 0,
        v1: seed + P64_1 + P64_2,
        v2: seed + P64_2,
        v3: seed,
        v4: seed - P64_1
    };
    if (input.length < 32) {
        state.u8a.set(input);
        state.u8asize = input.length;
        return state;
    }
    const limit = input.length - 32;
    let p = 0;
    if (limit >= 0) {
        const adjustV = (v) => P64_1 * rotl(v + P64_2 * fromU8a(input, p, 4), _31n);
        do {
            state.v1 = adjustV(state.v1);
            p += 8;
            state.v2 = adjustV(state.v2);
            p += 8;
            state.v3 = adjustV(state.v3);
            p += 8;
            state.v4 = adjustV(state.v4);
            p += 8;
        } while (p <= limit);
    }
    if (p < input.length) {
        state.u8a.set(input.subarray(p, input.length));
        state.u8asize = input.length - p;
    }
    return state;
}
function xxhash64(input, initSeed) {
    const { seed, u8a, u8asize, v1, v2, v3, v4 } = init((0, x_bigint_1.BigInt)(initSeed), input);
    let p = 0;
    let h64 = U64 & ((0, x_bigint_1.BigInt)(input.length) + (input.length >= 32
        ? (((((((((rotl(v1, util_1._1n) + rotl(v2, _7n) + rotl(v3, _12n) + rotl(v4, _18n)) ^ (P64_1 * rotl(v1 * P64_2, _31n))) * P64_1 + P64_4) ^ (P64_1 * rotl(v2 * P64_2, _31n))) * P64_1 + P64_4) ^ (P64_1 * rotl(v3 * P64_2, _31n))) * P64_1 + P64_4) ^ (P64_1 * rotl(v4 * P64_2, _31n))) * P64_1 + P64_4)
        : (seed + P64_5)));
    while (p <= (u8asize - 8)) {
        h64 = U64 & (P64_4 + P64_1 * rotl(h64 ^ (P64_1 * rotl(P64_2 * fromU8a(u8a, p, 4), _31n)), _27n));
        p += 8;
    }
    if ((p + 4) <= u8asize) {
        h64 = U64 & (P64_3 + P64_2 * rotl(h64 ^ (P64_1 * fromU8a(u8a, p, 2)), _23n));
        p += 4;
    }
    while (p < u8asize) {
        h64 = U64 & (P64_1 * rotl(h64 ^ (P64_5 * (0, x_bigint_1.BigInt)(u8a[p++])), _11n));
    }
    h64 = U64 & (P64_2 * (h64 ^ (h64 >> _33n)));
    h64 = U64 & (P64_3 * (h64 ^ (h64 >> _29n)));
    h64 = U64 & (h64 ^ (h64 >> _32n));
    const result = new Uint8Array(8);
    for (let i = 7; i >= 0; i--) {
        result[i] = Number(h64 % _256n);
        h64 = h64 / _256n;
    }
    return result;
}
