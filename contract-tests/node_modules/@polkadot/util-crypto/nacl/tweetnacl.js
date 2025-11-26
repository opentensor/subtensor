/* eslint-disable brace-style,camelcase,comma-spacing,curly,one-var,padding-line-between-statements,space-infix-ops */
function L32(x, c) { return (x << c) | (x >>> (32 - c)); }
function ld32(x, i) {
    let u = x[i + 3] & 0xff;
    u = (u << 8) | (x[i + 2] & 0xff);
    u = (u << 8) | (x[i + 1] & 0xff);
    return (u << 8) | (x[i + 0] & 0xff);
}
function st32(x, j, u) {
    for (let i = 0; i < 4; i++) {
        x[j + i] = u & 255;
        u >>>= 8;
    }
}
function vn(x, xi, y, yi, n) {
    let d = 0;
    for (let i = 0; i < n; i++)
        d |= x[xi + i] ^ y[yi + i];
    return (1 & ((d - 1) >>> 8)) - 1;
}
function core(out, inp, k, c, h) {
    const w = new Uint32Array(16), x = new Uint32Array(16), y = new Uint32Array(16), t = new Uint32Array(4);
    let i, j, m;
    for (i = 0; i < 4; i++) {
        x[5 * i] = ld32(c, 4 * i);
        x[1 + i] = ld32(k, 4 * i);
        x[6 + i] = ld32(inp, 4 * i);
        x[11 + i] = ld32(k, 16 + 4 * i);
    }
    for (i = 0; i < 16; i++)
        y[i] = x[i];
    for (i = 0; i < 20; i++) {
        for (j = 0; j < 4; j++) {
            for (m = 0; m < 4; m++)
                t[m] = x[(5 * j + 4 * m) % 16];
            t[1] ^= L32((t[0] + t[3]) | 0, 7);
            t[2] ^= L32((t[1] + t[0]) | 0, 9);
            t[3] ^= L32((t[2] + t[1]) | 0, 13);
            t[0] ^= L32((t[3] + t[2]) | 0, 18);
            for (m = 0; m < 4; m++)
                w[4 * j + (j + m) % 4] = t[m];
        }
        for (m = 0; m < 16; m++)
            x[m] = w[m];
    }
    if (h) {
        for (i = 0; i < 16; i++)
            x[i] = (x[i] + y[i]) | 0;
        for (i = 0; i < 4; i++) {
            x[5 * i] = (x[5 * i] - ld32(c, 4 * i)) | 0;
            x[6 + i] = (x[6 + i] - ld32(inp, 4 * i)) | 0;
        }
        for (i = 0; i < 4; i++) {
            st32(out, 4 * i, x[5 * i]);
            st32(out, 16 + 4 * i, x[6 + i]);
        }
    }
    else {
        for (i = 0; i < 16; i++)
            st32(out, 4 * i, (x[i] + y[i]) | 0);
    }
}
const sigma = new Uint8Array([101, 120, 112, 97, 110, 100, 32, 51, 50, 45, 98, 121, 116, 101, 32, 107]);
function crypto_stream_salsa20_xor(c, cpos, m, mpos, b, n, k) {
    const z = new Uint8Array(16), x = new Uint8Array(64);
    let u, i;
    if (!b)
        return 0;
    for (i = 0; i < 16; i++)
        z[i] = 0;
    for (i = 0; i < 8; i++)
        z[i] = n[i];
    while (b >= 64) {
        core(x, z, k, sigma, false);
        for (i = 0; i < 64; i++)
            c[cpos + i] = (m ? m[mpos + i] : 0) ^ x[i];
        u = 1;
        for (i = 8; i < 16; i++) {
            u = u + (z[i] & 0xff) | 0;
            z[i] = u & 0xff;
            u >>>= 8;
        }
        b -= 64;
        cpos += 64;
        if (m)
            mpos += 64;
    }
    if (b > 0) {
        core(x, z, k, sigma, false);
        for (i = 0; i < b; i++)
            c[cpos + i] = (m ? m[mpos + i] : 0) ^ x[i];
    }
    return 0;
}
function crypto_stream_xor(c, cpos, m, mpos, d, n, k) {
    const s = new Uint8Array(32);
    core(s, n, k, sigma, true);
    return crypto_stream_salsa20_xor(c, cpos, m, mpos, d, n.subarray(16), s);
}
function add1305(h, c) {
    let u = 0;
    for (let j = 0; j < 17; j++) {
        u = (u + ((h[j] + c[j]) | 0)) | 0;
        h[j] = u & 255;
        u >>>= 8;
    }
}
const minusp = new Uint32Array([5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 252]);
function crypto_onetimeauth(out, outpos, m, mpos, n, k) {
    let i, j, u;
    const x = new Uint32Array(17), r = new Uint32Array(17), h = new Uint32Array(17), c = new Uint32Array(17), g = new Uint32Array(17);
    for (j = 0; j < 17; j++)
        r[j] = h[j] = 0;
    for (j = 0; j < 16; j++)
        r[j] = k[j];
    r[3] &= 15;
    r[4] &= 252;
    r[7] &= 15;
    r[8] &= 252;
    r[11] &= 15;
    r[12] &= 252;
    r[15] &= 15;
    while (n > 0) {
        for (j = 0; j < 17; j++)
            c[j] = 0;
        for (j = 0; (j < 16) && (j < n); ++j)
            c[j] = m[mpos + j];
        c[j] = 1;
        mpos += j;
        n -= j;
        add1305(h, c);
        for (i = 0; i < 17; i++) {
            x[i] = 0;
            for (j = 0; j < 17; j++)
                x[i] = (x[i] + (h[j] * ((j <= i) ? r[i - j] : ((320 * r[i + 17 - j]) | 0))) | 0) | 0;
        }
        for (i = 0; i < 17; i++)
            h[i] = x[i];
        u = 0;
        for (j = 0; j < 16; j++) {
            u = (u + h[j]) | 0;
            h[j] = u & 255;
            u >>>= 8;
        }
        u = (u + h[16]) | 0;
        h[16] = u & 3;
        u = (5 * (u >>> 2)) | 0;
        for (j = 0; j < 16; j++) {
            u = (u + h[j]) | 0;
            h[j] = u & 255;
            u >>>= 8;
        }
        u = (u + h[16]) | 0;
        h[16] = u;
    }
    for (j = 0; j < 17; j++)
        g[j] = h[j];
    add1305(h, minusp);
    const s = (-(h[16] >>> 7) | 0);
    for (j = 0; j < 17; j++)
        h[j] ^= s & (g[j] ^ h[j]);
    for (j = 0; j < 16; j++)
        c[j] = k[j + 16];
    c[16] = 0;
    add1305(h, c);
    for (j = 0; j < 16; j++)
        out[outpos + j] = h[j];
    return 0;
}
function crypto_onetimeauth_verify(h, hpos, m, mpos, n, k) {
    const x = new Uint8Array(16);
    crypto_onetimeauth(x, 0, m, mpos, n, k);
    return vn(h, hpos, x, 0, 16);
}
function crypto_secretbox(c, m, d, n, k) {
    if (d < 32)
        return -1;
    crypto_stream_xor(c, 0, m, 0, d, n, k);
    crypto_onetimeauth(c, 16, c, 32, d - 32, c);
    for (let i = 0; i < 16; i++)
        c[i] = 0;
    return 0;
}
function crypto_secretbox_open(m, c, d, n, k) {
    const x = new Uint8Array(32);
    if (d < 32)
        return -1;
    crypto_stream_xor(x, 0, null, 0, 32, n, k);
    if (crypto_onetimeauth_verify(c, 16, c, 32, d - 32, x) !== 0)
        return -1;
    crypto_stream_xor(m, 0, c, 0, d, n, k);
    for (let i = 0; i < 32; i++)
        m[i] = 0;
    return 0;
}
const crypto_secretbox_KEYBYTES = 32;
const crypto_secretbox_NONCEBYTES = 24;
const crypto_secretbox_ZEROBYTES = 32;
const crypto_secretbox_BOXZEROBYTES = 16;
function checkLengths(k, n) {
    if (k.length !== crypto_secretbox_KEYBYTES)
        throw new Error('bad key size');
    if (n.length !== crypto_secretbox_NONCEBYTES)
        throw new Error('bad nonce size');
}
function checkArrayTypes(...args) {
    for (let i = 0, count = args.length; i < count; i++) {
        if (!(args[i] instanceof Uint8Array))
            throw new TypeError('unexpected type, use Uint8Array');
    }
}
export function naclSecretbox(msg, nonce, key) {
    checkArrayTypes(msg, nonce, key);
    checkLengths(key, nonce);
    const m = new Uint8Array(crypto_secretbox_ZEROBYTES + msg.length);
    const c = new Uint8Array(m.length);
    for (let i = 0; i < msg.length; i++)
        m[i + crypto_secretbox_ZEROBYTES] = msg[i];
    crypto_secretbox(c, m, m.length, nonce, key);
    return c.subarray(crypto_secretbox_BOXZEROBYTES);
}
export function naclSecretboxOpen(box, nonce, key) {
    checkArrayTypes(box, nonce, key);
    checkLengths(key, nonce);
    const c = new Uint8Array(crypto_secretbox_BOXZEROBYTES + box.length);
    const m = new Uint8Array(c.length);
    for (let i = 0; i < box.length; i++)
        c[i + crypto_secretbox_BOXZEROBYTES] = box[i];
    if (c.length < 32)
        return null;
    if (crypto_secretbox_open(m, c, c.length, nonce, key) !== 0)
        return null;
    return m.subarray(crypto_secretbox_ZEROBYTES);
}
