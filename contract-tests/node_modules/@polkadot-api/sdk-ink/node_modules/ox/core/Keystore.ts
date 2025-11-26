import { ctr } from '@noble/ciphers/aes'
import {
  pbkdf2 as pbkdf2_noble,
  pbkdf2Async as pbkdf2Async_noble,
} from '@noble/hashes/pbkdf2'
import {
  scrypt as scrypt_noble,
  scryptAsync as scryptAsync_noble,
} from '@noble/hashes/scrypt'
import { sha256 } from '@noble/hashes/sha2'
import * as Bytes from './Bytes.js'
import type * as Errors from './Errors.js'
import * as Hash from './Hash.js'
import type * as Hex from './Hex.js'
import type { OneOf } from './internal/types.js'

/** Base Derivation Options. */
type BaseDeriveOpts<
  kdf extends string = string,
  kdfparams extends Record<string, unknown> = Record<string, unknown>,
> = {
  iv: Bytes.Bytes
  kdfparams: kdfparams
  kdf: kdf
}

/** Keystore. */
export type Keystore = {
  crypto: {
    cipher: 'aes-128-ctr'
    ciphertext: string
    cipherparams: {
      iv: string
    }
    mac: string
  } & Pick<DeriveOpts, 'kdf' | 'kdfparams'>
  id: string
  version: 3
}

/** Key. */
export type Key = (() => Hex.Hex) | Hex.Hex

/** Derivation Options. */
export type DeriveOpts = Pbkdf2DeriveOpts | ScryptDeriveOpts

/** PBKDF2 Derivation Options. */
export type Pbkdf2DeriveOpts = BaseDeriveOpts<
  'pbkdf2',
  {
    c: number
    dklen: number
    prf: 'hmac-sha256'
    salt: string
  }
>

/** Scrypt Derivation Options. */
export type ScryptDeriveOpts = BaseDeriveOpts<
  'scrypt',
  {
    dklen: number
    n: number
    p: number
    r: number
    salt: string
  }
>

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
export function decrypt<as extends 'Hex' | 'Bytes' = 'Hex'>(
  keystore: Keystore,
  key: Key,
  options: decrypt.Options<as> = {},
): decrypt.ReturnType<as> {
  const { as = 'Hex' } = options
  const key_ = Bytes.from(typeof key === 'function' ? key() : key)

  const encKey = Bytes.slice(key_, 0, 16)
  const macKey = Bytes.slice(key_, 16, 32)

  const ciphertext = Bytes.from(`0x${keystore.crypto.ciphertext}`)
  const mac = Hash.keccak256(Bytes.concat(macKey, ciphertext))

  if (!Bytes.isEqual(mac, Bytes.from(`0x${keystore.crypto.mac}`)))
    throw new Error('corrupt keystore')

  const data = ctr(
    encKey,
    Bytes.from(`0x${keystore.crypto.cipherparams.iv}`),
  ).decrypt(ciphertext)

  if (as === 'Hex') return Bytes.toHex(data) as never
  return data as never
}

export declare namespace decrypt {
  type Options<as extends 'Hex' | 'Bytes' = 'Hex' | 'Bytes'> = {
    /** Output format. @default 'Hex' */
    as?: as | 'Hex' | 'Bytes' | undefined
  }

  type ReturnType<as extends 'Hex' | 'Bytes' = 'Hex' | 'Bytes'> =
    | (as extends 'Hex' ? Hex.Hex : never)
    | (as extends 'Bytes' ? Bytes.Bytes : never)
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
export function encrypt(
  privateKey: Bytes.Bytes | Hex.Hex,
  key: Key,
  options: encrypt.Options,
): Keystore {
  const { id = crypto.randomUUID(), kdf, kdfparams, iv } = options

  const key_ = Bytes.from(typeof key === 'function' ? key() : key)
  const value_ = Bytes.from(privateKey)

  const encKey = Bytes.slice(key_, 0, 16)
  const macKey = Bytes.slice(key_, 16, 32)

  const ciphertext = ctr(encKey, iv).encrypt(value_)
  const mac = Hash.keccak256(Bytes.concat(macKey, ciphertext))

  return {
    crypto: {
      cipher: 'aes-128-ctr',
      ciphertext: Bytes.toHex(ciphertext).slice(2),
      cipherparams: { iv: Bytes.toHex(iv).slice(2) },
      kdf,
      kdfparams,
      mac: Bytes.toHex(mac).slice(2),
    } as Keystore['crypto'],
    id,
    version: 3,
  }
}

export declare namespace encrypt {
  type Options = DeriveOpts & {
    /** UUID. */
    id?: string | undefined
  }
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
export function pbkdf2(options: pbkdf2.Options) {
  const { iv, iterations = 262_144, password } = options

  const salt = options.salt ? Bytes.from(options.salt) : Bytes.random(32)
  const key = Bytes.toHex(
    pbkdf2_noble(sha256, password, salt, { c: iterations, dkLen: 32 }),
  )

  return defineKey(() => key, {
    iv,
    kdfparams: {
      c: iterations,
      dklen: 32,
      prf: 'hmac-sha256',
      salt: Bytes.toHex(salt).slice(2),
    },
    kdf: 'pbkdf2',
  }) satisfies [Key, Pbkdf2DeriveOpts]
}

export declare namespace pbkdf2 {
  type Options = {
    /** The counter to use for the AES-CTR encryption. */
    iv?: Bytes.Bytes | Hex.Hex | undefined
    /** The number of iterations to use. @default 262_144 */
    iterations?: number | undefined
    /** Password to derive key from. */
    password: string
    /** Salt to use for key derivation. @default `Bytes.random(32)` */
    salt?: Bytes.Bytes | Hex.Hex | undefined
  }
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
export async function pbkdf2Async(options: pbkdf2.Options) {
  const { iv, iterations = 262_144, password } = options

  const salt = options.salt ? Bytes.from(options.salt) : Bytes.random(32)
  const key = Bytes.toHex(
    await pbkdf2Async_noble(sha256, password, salt, {
      c: iterations,
      dkLen: 32,
    }),
  )

  return defineKey(() => key, {
    iv,
    kdfparams: {
      c: iterations,
      dklen: 32,
      prf: 'hmac-sha256',
      salt: Bytes.toHex(salt).slice(2),
    },
    kdf: 'pbkdf2',
  }) satisfies [Key, Pbkdf2DeriveOpts]
}

export declare namespace pbkdf2Async {
  type Options = pbkdf2.Options
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
export function scrypt(options: scrypt.Options) {
  const { iv, n = 262_144, password, p = 8, r = 1 } = options

  const salt = options.salt ? Bytes.from(options.salt) : Bytes.random(32)
  const key = Bytes.toHex(
    scrypt_noble(password, salt, { N: n, dkLen: 32, r, p }),
  )

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
  }) satisfies [Key, ScryptDeriveOpts]
}

export declare namespace scrypt {
  type Options = {
    /** The counter to use for the AES-CTR encryption. */
    iv?: Bytes.Bytes | Hex.Hex | undefined
    /** Cost factor. @default 262_144 */
    n?: number | undefined
    /** Parallelization factor. @default 8 */
    p?: number | undefined
    /** Block size. @default 1 */
    r?: number | undefined
    /** Password to derive key from. */
    password: string
    /** Salt to use for key derivation. @default `Bytes.random(32)` */
    salt?: Bytes.Bytes | Hex.Hex | undefined
  }
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
export async function scryptAsync(options: scrypt.Options) {
  const { iv, n = 262_144, password } = options

  const p = 8
  const r = 1

  const salt = options.salt ? Bytes.from(options.salt) : Bytes.random(32)
  const key = Bytes.toHex(
    await scryptAsync_noble(password, salt, { N: n, dkLen: 32, r, p }),
  )

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
  }) satisfies [Key, ScryptDeriveOpts]
}

export declare namespace scryptAsync {
  type Options = scrypt.Options
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
export function toKey(keystore: Keystore, options: toKey.Options): Key {
  const { crypto } = keystore
  const { password } = options
  const { cipherparams, kdf, kdfparams } = crypto
  const { iv } = cipherparams
  const { c, n, p, r, salt } = kdfparams as OneOf<
    Pbkdf2DeriveOpts['kdfparams'] | ScryptDeriveOpts['kdfparams']
  >

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
        })
      case 'pbkdf2':
        return pbkdf2({
          iv: Bytes.from(`0x${iv}`),
          iterations: c,
          password,
          salt: Bytes.from(`0x${salt}`),
        })
      default:
        throw new Error('unsupported kdf')
    }
  })()

  return key
}

export declare namespace toKey {
  type Options = {
    /** Password to derive key from. */
    password: string
  }
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
export async function toKeyAsync(
  keystore: Keystore,
  options: toKeyAsync.Options,
): Promise<Key> {
  const { crypto } = keystore
  const { password } = options
  const { cipherparams, kdf, kdfparams } = crypto
  const { iv } = cipherparams
  const { c, n, p, r, salt } = kdfparams as OneOf<
    Pbkdf2DeriveOpts['kdfparams'] | ScryptDeriveOpts['kdfparams']
  >

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
        })
      case 'pbkdf2':
        return await pbkdf2({
          iv: Bytes.from(`0x${iv}`),
          iterations: c,
          password,
          salt: Bytes.from(`0x${salt}`),
        })
      default:
        throw new Error('unsupported kdf')
    }
  })()

  return key
}

export declare namespace toKeyAsync {
  type Options = {
    /** Password to derive key from. */
    password: string
  }
}

///////////////////////////////////////////////////////////////////////////

/** @internal */
// biome-ignore lint/correctness/noUnusedVariables: _
function defineKey<
  const key extends Key,
  const options extends defineKey.Options,
>(key: key, options: options): [key, options & { iv: Bytes.Bytes }] {
  const iv = options.iv ? Bytes.from(options.iv) : Bytes.random(16)
  return [key, { ...options, iv }] as never
}

/** @internal */
declare namespace defineKey {
  type Options<
    kdf extends string = string,
    kdfparams extends Record<string, unknown> = Record<string, unknown>,
  > = Omit<BaseDeriveOpts<kdf, kdfparams>, 'iv'> & {
    iv?: Bytes.Bytes | Hex.Hex | undefined
  }

  type ErrorType = Errors.GlobalErrorType
}
