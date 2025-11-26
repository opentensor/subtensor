"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.decrypt = decrypt;
exports.encrypt = encrypt;
exports.pbkdf2 = pbkdf2;
exports.pbkdf2Async = pbkdf2Async;
exports.scrypt = scrypt;
exports.scryptAsync = scryptAsync;
exports.toKey = toKey;
exports.toKeyAsync = toKeyAsync;
const aes_1 = require("@noble/ciphers/aes");
const pbkdf2_1 = require("@noble/hashes/pbkdf2");
const scrypt_1 = require("@noble/hashes/scrypt");
const sha2_1 = require("@noble/hashes/sha2");
const Bytes = require("./Bytes.js");
const Hash = require("./Hash.js");
function decrypt(keystore, key, options = {}) {
    const { as = 'Hex' } = options;
    const key_ = Bytes.from(typeof key === 'function' ? key() : key);
    const encKey = Bytes.slice(key_, 0, 16);
    const macKey = Bytes.slice(key_, 16, 32);
    const ciphertext = Bytes.from(`0x${keystore.crypto.ciphertext}`);
    const mac = Hash.keccak256(Bytes.concat(macKey, ciphertext));
    if (!Bytes.isEqual(mac, Bytes.from(`0x${keystore.crypto.mac}`)))
        throw new Error('corrupt keystore');
    const data = (0, aes_1.ctr)(encKey, Bytes.from(`0x${keystore.crypto.cipherparams.iv}`)).decrypt(ciphertext);
    if (as === 'Hex')
        return Bytes.toHex(data);
    return data;
}
function encrypt(privateKey, key, options) {
    const { id = crypto.randomUUID(), kdf, kdfparams, iv } = options;
    const key_ = Bytes.from(typeof key === 'function' ? key() : key);
    const value_ = Bytes.from(privateKey);
    const encKey = Bytes.slice(key_, 0, 16);
    const macKey = Bytes.slice(key_, 16, 32);
    const ciphertext = (0, aes_1.ctr)(encKey, iv).encrypt(value_);
    const mac = Hash.keccak256(Bytes.concat(macKey, ciphertext));
    return {
        crypto: {
            cipher: 'aes-128-ctr',
            ciphertext: Bytes.toHex(ciphertext).slice(2),
            cipherparams: { iv: Bytes.toHex(iv).slice(2) },
            kdf,
            kdfparams,
            mac: Bytes.toHex(mac).slice(2),
        },
        id,
        version: 3,
    };
}
function pbkdf2(options) {
    const { iv, iterations = 262_144, password } = options;
    const salt = options.salt ? Bytes.from(options.salt) : Bytes.random(32);
    const key = Bytes.toHex((0, pbkdf2_1.pbkdf2)(sha2_1.sha256, password, salt, { c: iterations, dkLen: 32 }));
    return defineKey(() => key, {
        iv,
        kdfparams: {
            c: iterations,
            dklen: 32,
            prf: 'hmac-sha256',
            salt: Bytes.toHex(salt).slice(2),
        },
        kdf: 'pbkdf2',
    });
}
async function pbkdf2Async(options) {
    const { iv, iterations = 262_144, password } = options;
    const salt = options.salt ? Bytes.from(options.salt) : Bytes.random(32);
    const key = Bytes.toHex(await (0, pbkdf2_1.pbkdf2Async)(sha2_1.sha256, password, salt, {
        c: iterations,
        dkLen: 32,
    }));
    return defineKey(() => key, {
        iv,
        kdfparams: {
            c: iterations,
            dklen: 32,
            prf: 'hmac-sha256',
            salt: Bytes.toHex(salt).slice(2),
        },
        kdf: 'pbkdf2',
    });
}
function scrypt(options) {
    const { iv, n = 262_144, password, p = 8, r = 1 } = options;
    const salt = options.salt ? Bytes.from(options.salt) : Bytes.random(32);
    const key = Bytes.toHex((0, scrypt_1.scrypt)(password, salt, { N: n, dkLen: 32, r, p }));
    return defineKey(() => key, {
        iv,
        kdfparams: {
            dklen: 32,
            n,
            p,
            r,
            salt: Bytes.toHex(salt).slice(2),
        },
        kdf: 'scrypt',
    });
}
async function scryptAsync(options) {
    const { iv, n = 262_144, password } = options;
    const p = 8;
    const r = 1;
    const salt = options.salt ? Bytes.from(options.salt) : Bytes.random(32);
    const key = Bytes.toHex(await (0, scrypt_1.scryptAsync)(password, salt, { N: n, dkLen: 32, r, p }));
    return defineKey(() => key, {
        iv,
        kdfparams: {
            dklen: 32,
            n,
            p,
            r,
            salt: Bytes.toHex(salt).slice(2),
        },
        kdf: 'scrypt',
    });
}
function toKey(keystore, options) {
    const { crypto } = keystore;
    const { password } = options;
    const { cipherparams, kdf, kdfparams } = crypto;
    const { iv } = cipherparams;
    const { c, n, p, r, salt } = kdfparams;
    const [key] = (() => {
        switch (kdf) {
            case 'scrypt':
                return scrypt({
                    iv: Bytes.from(`0x${iv}`),
                    n,
                    p,
                    r,
                    salt: Bytes.from(`0x${salt}`),
                    password,
                });
            case 'pbkdf2':
                return pbkdf2({
                    iv: Bytes.from(`0x${iv}`),
                    iterations: c,
                    password,
                    salt: Bytes.from(`0x${salt}`),
                });
            default:
                throw new Error('unsupported kdf');
        }
    })();
    return key;
}
async function toKeyAsync(keystore, options) {
    const { crypto } = keystore;
    const { password } = options;
    const { cipherparams, kdf, kdfparams } = crypto;
    const { iv } = cipherparams;
    const { c, n, p, r, salt } = kdfparams;
    const [key] = await (async () => {
        switch (kdf) {
            case 'scrypt':
                return await scryptAsync({
                    iv: Bytes.from(`0x${iv}`),
                    n,
                    p,
                    r,
                    salt: Bytes.from(`0x${salt}`),
                    password,
                });
            case 'pbkdf2':
                return await pbkdf2({
                    iv: Bytes.from(`0x${iv}`),
                    iterations: c,
                    password,
                    salt: Bytes.from(`0x${salt}`),
                });
            default:
                throw new Error('unsupported kdf');
        }
    })();
    return key;
}
function defineKey(key, options) {
    const iv = options.iv ? Bytes.from(options.iv) : Bytes.random(16);
    return [key, { ...options, iv }];
}
//# sourceMappingURL=Keystore.js.map