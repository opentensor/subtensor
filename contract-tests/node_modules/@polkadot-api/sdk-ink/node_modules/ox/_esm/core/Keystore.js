import { ctr } from '@noble/ciphers/aes';
import { pbkdf2 as pbkdf2_noble, pbkdf2Async as pbkdf2Async_noble, } from '@noble/hashes/pbkdf2';
import { scrypt as scrypt_noble, scryptAsync as scryptAsync_noble, } from '@noble/hashes/scrypt';
import { sha256 } from '@noble/hashes/sha2';
import * as Bytes from './Bytes.js';
import * as Hash from './Hash.js';
/**
 * Decrypts a [JSON keystore](https://ethereum.org/en/developers/docs/data-structures-and-encoding/web3-secret-storage/)
 * into a private key.
 *
 * Supports the following key derivation functions (KDFs):
 * - {@link ox#Keystore.(pbkdf2:function)}
 * - {@link ox#Keystore.(scrypt:function)}
 *
 * @example
 * ```ts twoslash
 * // @noErrors
 * import { Keystore, Secp256k1 } from 'ox'
 *
 * // JSON keystore.
 * const keystore = { crypto: { ... }, id: '...', version: 3 }
 *
 * // Derive the key using your password.
 * const key = Keystore.toKey(keystore, { password: 'hunter2' })
 *
 * // Decrypt the private key.
 * const privateKey = Keystore.decrypt(keystore, key)
 * // @log: "0x..."
 * ```
 *
 * @param keystore - JSON keystore.
 * @param key - Key to use for decryption.
 * @param options - Decryption options.
 * @returns Decrypted private key.
 */
export function decrypt(keystore, key, options = {}) {
    const { as = 'Hex' } = options;
    const key_ = Bytes.from(typeof key === 'function' ? key() : key);
    const encKey = Bytes.slice(key_, 0, 16);
    const macKey = Bytes.slice(key_, 16, 32);
    const ciphertext = Bytes.from(`0x${keystore.crypto.ciphertext}`);
    const mac = Hash.keccak256(Bytes.concat(macKey, ciphertext));
    if (!Bytes.isEqual(mac, Bytes.from(`0x${keystore.crypto.mac}`)))
        throw new Error('corrupt keystore');
    const data = ctr(encKey, Bytes.from(`0x${keystore.crypto.cipherparams.iv}`)).decrypt(ciphertext);
    if (as === 'Hex')
        return Bytes.toHex(data);
    return data;
}
/**
 * Encrypts a private key as a [JSON keystore](https://ethereum.org/en/developers/docs/data-structures-and-encoding/web3-secret-storage/)
 * using a derived key.
 *
 * Supports the following key derivation functions (KDFs):
 * - {@link ox#Keystore.(pbkdf2:function)}
 * - {@link ox#Keystore.(scrypt:function)}
 *
 * @example
 * ```ts twoslash
 * import { Keystore, Secp256k1 } from 'ox'
 *
 * // Generate a random private key.
 * const privateKey = Secp256k1.randomPrivateKey()
 *
 * // Derive key from password.
 * const [key, opts] = Keystore.pbkdf2({ password: 'testpassword' })
 *
 * // Encrypt the private key.
 * const encrypted = Keystore.encrypt(privateKey, key, opts)
 * // @log: {
 * // @log:   "crypto": {
 * // @log:     "cipher": "aes-128-ctr",
 * // @log:     "ciphertext": "...",
 * // @log:     "cipherparams": {
 * // @log:       "iv": "...",
 * // @log:     },
 * // @log:     "kdf": "pbkdf2",
 * // @log:     "kdfparams": {
 * // @log:       "salt": "...",
 * // @log:       "dklen": 32,
 * // @log:       "prf": "hmac-sha256",
 * // @log:       "c": 262144,
 * // @log:     },
 * // @log:     "mac": "...",
 * // @log:   },
 * // @log:   "id": "...",
 * // @log:   "version": 3,
 * // @log: }
 * ```
 *
 * @param privateKey - Private key to encrypt.
 * @param key - Key to use for encryption.
 * @param options - Encryption options.
 * @returns Encrypted keystore.
 */
export function encrypt(privateKey, key, options) {
    const { id = crypto.randomUUID(), kdf, kdfparams, iv } = options;
    const key_ = Bytes.from(typeof key === 'function' ? key() : key);
    const value_ = Bytes.from(privateKey);
    const encKey = Bytes.slice(key_, 0, 16);
    const macKey = Bytes.slice(key_, 16, 32);
    const ciphertext = ctr(encKey, iv).encrypt(value_);
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
/**
 * Derives a key from a password using [PBKDF2](https://en.wikipedia.org/wiki/PBKDF2).
 *
 * @example
 * ```ts twoslash
 * import { Keystore } from 'ox'
 *
 * const [key, opts] = Keystore.pbkdf2({ password: 'testpassword' })
 * ```
 *
 * @param options - PBKDF2 options.
 * @returns PBKDF2 key.
 */
export function pbkdf2(options) {
    const { iv, iterations = 262_144, password } = options;
    const salt = options.salt ? Bytes.from(options.salt) : Bytes.random(32);
    const key = Bytes.toHex(pbkdf2_noble(sha256, password, salt, { c: iterations, dkLen: 32 }));
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
/**
 * Derives a key from a password using [PBKDF2](https://en.wikipedia.org/wiki/PBKDF2).
 *
 * @example
 * ```ts twoslash
 * import { Keystore } from 'ox'
 *
 * const [key, opts] = await Keystore.pbkdf2Async({ password: 'testpassword' })
 * ```
 *
 * @param options - PBKDF2 options.
 * @returns PBKDF2 key.
 */
export async function pbkdf2Async(options) {
    const { iv, iterations = 262_144, password } = options;
    const salt = options.salt ? Bytes.from(options.salt) : Bytes.random(32);
    const key = Bytes.toHex(await pbkdf2Async_noble(sha256, password, salt, {
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
/**
 * Derives a key from a password using [scrypt](https://en.wikipedia.org/wiki/Scrypt).
 *
 * @example
 * ```ts twoslash
 * import { Keystore } from 'ox'
 *
 * const [key, opts] = Keystore.scrypt({ password: 'testpassword' })
 * ```
 *
 * @param options - Scrypt options.
 * @returns Scrypt key.
 */
export function scrypt(options) {
    const { iv, n = 262_144, password, p = 8, r = 1 } = options;
    const salt = options.salt ? Bytes.from(options.salt) : Bytes.random(32);
    const key = Bytes.toHex(scrypt_noble(password, salt, { N: n, dkLen: 32, r, p }));
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
/**
 * Derives a key from a password using [scrypt](https://en.wikipedia.org/wiki/Scrypt).
 *
 * @example
 * ```ts twoslash
 * import { Keystore } from 'ox'
 *
 * const [key, opts] = await Keystore.scryptAsync({ password: 'testpassword' })
 * ```
 *
 * @param options - Scrypt options.
 * @returns Scrypt key.
 */
export async function scryptAsync(options) {
    const { iv, n = 262_144, password } = options;
    const p = 8;
    const r = 1;
    const salt = options.salt ? Bytes.from(options.salt) : Bytes.random(32);
    const key = Bytes.toHex(await scryptAsync_noble(password, salt, { N: n, dkLen: 32, r, p }));
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
/**
 * Extracts a Key from a JSON Keystore to use for decryption.
 *
 * @example
 * ```ts twoslash
 * // @noErrors
 * import { Keystore } from 'ox'
 *
 * // JSON keystore.
 * const keystore = { crypto: { ... }, id: '...', version: 3 }
 *
 * const key = Keystore.toKey(keystore, { password: 'hunter2' }) // [!code focus]
 *
 * const decrypted = Keystore.decrypt(keystore, key)
 * ```
 *
 * @param keystore - JSON Keystore
 * @param options - Options
 * @returns Key
 */
export function toKey(keystore, options) {
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
/**
 * Extracts a Key asynchronously from a JSON Keystore to use for decryption.
 *
 * @example
 * ```ts twoslash
 * // @noErrors
 * import { Keystore } from 'ox'
 *
 * // JSON keystore.
 * const keystore = { crypto: { ... }, id: '...', version: 3 }
 *
 * const key = await Keystore.toKeyAsync(keystore, { password: 'hunter2' }) // [!code focus]
 *
 * const decrypted = Keystore.decrypt(keystore, key)
 * ```
 *
 * @param keystore - JSON Keystore
 * @param options - Options
 * @returns Key
 */
export async function toKeyAsync(keystore, options) {
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
///////////////////////////////////////////////////////////////////////////
/** @internal */
// biome-ignore lint/correctness/noUnusedVariables: _
function defineKey(key, options) {
    const iv = options.iv ? Bytes.from(options.iv) : Bytes.random(16);
    return [key, { ...options, iv }];
}
//# sourceMappingURL=Keystore.js.map