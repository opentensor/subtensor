import * as Bytes from './Bytes.js';
import type * as Hex from './Hex.js';
/** Base Derivation Options. */
type BaseDeriveOpts<kdf extends string = string, kdfparams extends Record<string, unknown> = Record<string, unknown>> = {
    iv: Bytes.Bytes;
    kdfparams: kdfparams;
    kdf: kdf;
};
/** Keystore. */
export type Keystore = {
    crypto: {
        cipher: 'aes-128-ctr';
        ciphertext: string;
        cipherparams: {
            iv: string;
        };
        mac: string;
    } & Pick<DeriveOpts, 'kdf' | 'kdfparams'>;
    id: string;
    version: 3;
};
/** Key. */
export type Key = (() => Hex.Hex) | Hex.Hex;
/** Derivation Options. */
export type DeriveOpts = Pbkdf2DeriveOpts | ScryptDeriveOpts;
/** PBKDF2 Derivation Options. */
export type Pbkdf2DeriveOpts = BaseDeriveOpts<'pbkdf2', {
    c: number;
    dklen: number;
    prf: 'hmac-sha256';
    salt: string;
}>;
/** Scrypt Derivation Options. */
export type ScryptDeriveOpts = BaseDeriveOpts<'scrypt', {
    dklen: number;
    n: number;
    p: number;
    r: number;
    salt: string;
}>;
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
export declare function decrypt<as extends 'Hex' | 'Bytes' = 'Hex'>(keystore: Keystore, key: Key, options?: decrypt.Options<as>): decrypt.ReturnType<as>;
export declare namespace decrypt {
    type Options<as extends 'Hex' | 'Bytes' = 'Hex' | 'Bytes'> = {
        /** Output format. @default 'Hex' */
        as?: as | 'Hex' | 'Bytes' | undefined;
    };
    type ReturnType<as extends 'Hex' | 'Bytes' = 'Hex' | 'Bytes'> = (as extends 'Hex' ? Hex.Hex : never) | (as extends 'Bytes' ? Bytes.Bytes : never);
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
export declare function encrypt(privateKey: Bytes.Bytes | Hex.Hex, key: Key, options: encrypt.Options): Keystore;
export declare namespace encrypt {
    type Options = DeriveOpts & {
        /** UUID. */
        id?: string | undefined;
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
export declare function pbkdf2(options: pbkdf2.Options): [() => `0x${string}`, {
    readonly iv: Uint8Array | `0x${string}` | undefined;
    readonly kdfparams: {
        readonly c: number;
        readonly dklen: 32;
        readonly prf: "hmac-sha256";
        readonly salt: string;
    };
    readonly kdf: "pbkdf2";
} & {
    iv: Bytes.Bytes;
}];
export declare namespace pbkdf2 {
    type Options = {
        /** The counter to use for the AES-CTR encryption. */
        iv?: Bytes.Bytes | Hex.Hex | undefined;
        /** The number of iterations to use. @default 262_144 */
        iterations?: number | undefined;
        /** Password to derive key from. */
        password: string;
        /** Salt to use for key derivation. @default `Bytes.random(32)` */
        salt?: Bytes.Bytes | Hex.Hex | undefined;
    };
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
export declare function pbkdf2Async(options: pbkdf2.Options): Promise<[() => `0x${string}`, {
    readonly iv: Uint8Array | `0x${string}` | undefined;
    readonly kdfparams: {
        readonly c: number;
        readonly dklen: 32;
        readonly prf: "hmac-sha256";
        readonly salt: string;
    };
    readonly kdf: "pbkdf2";
} & {
    iv: Bytes.Bytes;
}]>;
export declare namespace pbkdf2Async {
    type Options = pbkdf2.Options;
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
export declare function scrypt(options: scrypt.Options): [() => `0x${string}`, {
    readonly iv: Uint8Array | `0x${string}` | undefined;
    readonly kdfparams: {
        readonly dklen: 32;
        readonly n: number;
        readonly p: number;
        readonly r: number;
        readonly salt: string;
    };
    readonly kdf: "scrypt";
} & {
    iv: Bytes.Bytes;
}];
export declare namespace scrypt {
    type Options = {
        /** The counter to use for the AES-CTR encryption. */
        iv?: Bytes.Bytes | Hex.Hex | undefined;
        /** Cost factor. @default 262_144 */
        n?: number | undefined;
        /** Parallelization factor. @default 8 */
        p?: number | undefined;
        /** Block size. @default 1 */
        r?: number | undefined;
        /** Password to derive key from. */
        password: string;
        /** Salt to use for key derivation. @default `Bytes.random(32)` */
        salt?: Bytes.Bytes | Hex.Hex | undefined;
    };
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
export declare function scryptAsync(options: scrypt.Options): Promise<[() => `0x${string}`, {
    readonly iv: Uint8Array | `0x${string}` | undefined;
    readonly kdfparams: {
        readonly dklen: 32;
        readonly n: number;
        readonly p: 8;
        readonly r: 1;
        readonly salt: string;
    };
    readonly kdf: "scrypt";
} & {
    iv: Bytes.Bytes;
}]>;
export declare namespace scryptAsync {
    type Options = scrypt.Options;
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
export declare function toKey(keystore: Keystore, options: toKey.Options): Key;
export declare namespace toKey {
    type Options = {
        /** Password to derive key from. */
        password: string;
    };
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
export declare function toKeyAsync(keystore: Keystore, options: toKeyAsync.Options): Promise<Key>;
export declare namespace toKeyAsync {
    type Options = {
        /** Password to derive key from. */
        password: string;
    };
}
export {};
//# sourceMappingURL=Keystore.d.ts.map