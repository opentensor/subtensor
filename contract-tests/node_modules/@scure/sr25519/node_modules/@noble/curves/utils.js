/**
 * Hex, bytes and number utilities.
 * @module
 */
/*! noble-curves - MIT License (c) 2022 Paul Miller (paulmillr.com) */
import { abytes as abytes_, anumber, bytesToHex as bytesToHex_, concatBytes as concatBytes_, hexToBytes as hexToBytes_, } from '@noble/hashes/utils.js';
export { abytes, anumber, bytesToHex, concatBytes, hexToBytes, isBytes, randomBytes, } from '@noble/hashes/utils.js';
const _0n = /* @__PURE__ */ BigInt(0);
const _1n = /* @__PURE__ */ BigInt(1);
export function abool(value, title = '') {
    if (typeof value !== 'boolean') {
        const prefix = title && `"${title}" `;
        throw new Error(prefix + 'expected boolean, got type=' + typeof value);
    }
    return value;
}
// Used in weierstrass, der
function abignumber(n) {
    if (typeof n === 'bigint') {
        if (!isPosBig(n))
            throw new Error('positive bigint expected, got ' + n);
    }
    else
        anumber(n);
    return n;
}
export function asafenumber(value, title = '') {
    if (!Number.isSafeInteger(value)) {
        const prefix = title && `"${title}" `;
        throw new Error(prefix + 'expected safe integer, got type=' + typeof value);
    }
}
export function numberToHexUnpadded(num) {
    const hex = abignumber(num).toString(16);
    return hex.length & 1 ? '0' + hex : hex;
}
export function hexToNumber(hex) {
    if (typeof hex !== 'string')
        throw new Error('hex string expected, got ' + typeof hex);
    return hex === '' ? _0n : BigInt('0x' + hex); // Big Endian
}
// BE: Big Endian, LE: Little Endian
export function bytesToNumberBE(bytes) {
    return hexToNumber(bytesToHex_(bytes));
}
export function bytesToNumberLE(bytes) {
    return hexToNumber(bytesToHex_(copyBytes(abytes_(bytes)).reverse()));
}
export function numberToBytesBE(n, len) {
    anumber(len);
    n = abignumber(n);
    const res = hexToBytes_(n.toString(16).padStart(len * 2, '0'));
    if (res.length !== len)
        throw new Error('number too large');
    return res;
}
export function numberToBytesLE(n, len) {
    return numberToBytesBE(n, len).reverse();
}
// Unpadded, rarely used
export function numberToVarBytesBE(n) {
    return hexToBytes_(numberToHexUnpadded(abignumber(n)));
}
// Compares 2 u8a-s in kinda constant time
export function equalBytes(a, b) {
    if (a.length !== b.length)
        return false;
    let diff = 0;
    for (let i = 0; i < a.length; i++)
        diff |= a[i] ^ b[i];
    return diff === 0;
}
/**
 * Copies Uint8Array. We can't use u8a.slice(), because u8a can be Buffer,
 * and Buffer#slice creates mutable copy. Never use Buffers!
 */
export function copyBytes(bytes) {
    return Uint8Array.from(bytes);
}
/**
 * Decodes 7-bit ASCII string to Uint8Array, throws on non-ascii symbols
 * Should be safe to use for things expected to be ASCII.
 * Returns exact same result as `TextEncoder` for ASCII or throws.
 */
export function asciiToBytes(ascii) {
    return Uint8Array.from(ascii, (c, i) => {
        const charCode = c.charCodeAt(0);
        if (c.length !== 1 || charCode > 127) {
            throw new Error(`string contains non-ASCII character "${ascii[i]}" with code ${charCode} at position ${i}`);
        }
        return charCode;
    });
}
// Is positive bigint
const isPosBig = (n) => typeof n === 'bigint' && _0n <= n;
export function inRange(n, min, max) {
    return isPosBig(n) && isPosBig(min) && isPosBig(max) && min <= n && n < max;
}
/**
 * Asserts min <= n < max. NOTE: It's < max and not <= max.
 * @example
 * aInRange('x', x, 1n, 256n); // would assume x is in (1n..255n)
 */
export function aInRange(title, n, min, max) {
    // Why min <= n < max and not a (min < n < max) OR b (min <= n <= max)?
    // consider P=256n, min=0n, max=P
    // - a for min=0 would require -1:          `inRange('x', x, -1n, P)`
    // - b would commonly require subtraction:  `inRange('x', x, 0n, P - 1n)`
    // - our way is the cleanest:               `inRange('x', x, 0n, P)
    if (!inRange(n, min, max))
        throw new Error('expected valid ' + title + ': ' + min + ' <= n < ' + max + ', got ' + n);
}
// Bit operations
/**
 * Calculates amount of bits in a bigint.
 * Same as `n.toString(2).length`
 * TODO: merge with nLength in modular
 */
export function bitLen(n) {
    let len;
    for (len = 0; n > _0n; n >>= _1n, len += 1)
        ;
    return len;
}
/**
 * Gets single bit at position.
 * NOTE: first bit position is 0 (same as arrays)
 * Same as `!!+Array.from(n.toString(2)).reverse()[pos]`
 */
export function bitGet(n, pos) {
    return (n >> BigInt(pos)) & _1n;
}
/**
 * Sets single bit at position.
 */
export function bitSet(n, pos, value) {
    return n | ((value ? _1n : _0n) << BigInt(pos));
}
/**
 * Calculate mask for N bits. Not using ** operator with bigints because of old engines.
 * Same as BigInt(`0b${Array(i).fill('1').join('')}`)
 */
export const bitMask = (n) => (_1n << BigInt(n)) - _1n;
/**
 * Minimal HMAC-DRBG from NIST 800-90 for RFC6979 sigs.
 * @returns function that will call DRBG until 2nd arg returns something meaningful
 * @example
 *   const drbg = createHmacDRBG<Key>(32, 32, hmac);
 *   drbg(seed, bytesToKey); // bytesToKey must return Key or undefined
 */
export function createHmacDrbg(hashLen, qByteLen, hmacFn) {
    anumber(hashLen, 'hashLen');
    anumber(qByteLen, 'qByteLen');
    if (typeof hmacFn !== 'function')
        throw new Error('hmacFn must be a function');
    const u8n = (len) => new Uint8Array(len); // creates Uint8Array
    const NULL = Uint8Array.of();
    const byte0 = Uint8Array.of(0x00);
    const byte1 = Uint8Array.of(0x01);
    const _maxDrbgIters = 1000;
    // Step B, Step C: set hashLen to 8*ceil(hlen/8)
    let v = u8n(hashLen); // Minimal non-full-spec HMAC-DRBG from NIST 800-90 for RFC6979 sigs.
    let k = u8n(hashLen); // Steps B and C of RFC6979 3.2: set hashLen, in our case always same
    let i = 0; // Iterations counter, will throw when over 1000
    const reset = () => {
        v.fill(1);
        k.fill(0);
        i = 0;
    };
    const h = (...msgs) => hmacFn(k, concatBytes_(v, ...msgs)); // hmac(k)(v, ...values)
    const reseed = (seed = NULL) => {
        // HMAC-DRBG reseed() function. Steps D-G
        k = h(byte0, seed); // k = hmac(k || v || 0x00 || seed)
        v = h(); // v = hmac(k || v)
        if (seed.length === 0)
            return;
        k = h(byte1, seed); // k = hmac(k || v || 0x01 || seed)
        v = h(); // v = hmac(k || v)
    };
    const gen = () => {
        // HMAC-DRBG generate() function
        if (i++ >= _maxDrbgIters)
            throw new Error('drbg: tried max amount of iterations');
        let len = 0;
        const out = [];
        while (len < qByteLen) {
            v = h();
            const sl = v.slice();
            out.push(sl);
            len += v.length;
        }
        return concatBytes_(...out);
    };
    const genUntil = (seed, pred) => {
        reset();
        reseed(seed); // Steps D-G
        let res = undefined; // Step H: grind until k is in [1..n-1]
        while (!(res = pred(gen())))
            reseed();
        reset();
        return res;
    };
    return genUntil;
}
export function validateObject(object, fields = {}, optFields = {}) {
    if (!object || typeof object !== 'object')
        throw new Error('expected valid options object');
    function checkField(fieldName, expectedType, isOpt) {
        const val = object[fieldName];
        if (isOpt && val === undefined)
            return;
        const current = typeof val;
        if (current !== expectedType || val === null)
            throw new Error(`param "${fieldName}" is invalid: expected ${expectedType}, got ${current}`);
    }
    const iter = (f, isOpt) => Object.entries(f).forEach(([k, v]) => checkField(k, v, isOpt));
    iter(fields, false);
    iter(optFields, true);
}
/**
 * throws not implemented error
 */
export const notImplemented = () => {
    throw new Error('not implemented');
};
/**
 * Memoizes (caches) computation result.
 * Uses WeakMap: the value is going auto-cleaned by GC after last reference is removed.
 */
export function memoized(fn) {
    const map = new WeakMap();
    return (arg, ...args) => {
        const val = map.get(arg);
        if (val !== undefined)
            return val;
        const computed = fn(arg, ...args);
        map.set(arg, computed);
        return computed;
    };
}
//# sourceMappingURL=utils.js.map