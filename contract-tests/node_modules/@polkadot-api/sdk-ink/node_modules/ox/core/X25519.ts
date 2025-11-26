import { x25519 } from '@noble/curves/ed25519'
import * as Bytes from './Bytes.js'
import type * as Errors from './Errors.js'
import * as Hex from './Hex.js'

/** Re-export of noble/curves X25519 utilities. */
export const noble = x25519

/**
 * Creates a new X25519 key pair consisting of a private key and its corresponding public key.
 *
 * @example
 * ```ts twoslash
 * import { X25519 } from 'ox'
 *
 * const { privateKey, publicKey } = X25519.createKeyPair()
 * ```
 *
 * @param options - The options to generate the key pair.
 * @returns The generated key pair containing both private and public keys.
 */
export function createKeyPair<as extends 'Hex' | 'Bytes' = 'Hex'>(
  options: createKeyPair.Options<as> = {},
): createKeyPair.ReturnType<as> {
  const { as = 'Hex' } = options
  const privateKey = randomPrivateKey({ as })
  const publicKey = getPublicKey({ privateKey, as })

  return {
    privateKey: privateKey as never,
    publicKey: publicKey as never,
  }
}

export declare namespace createKeyPair {
  type Options<as extends 'Hex' | 'Bytes' = 'Hex'> = {
    /**
     * Format of the returned private and public keys.
     * @default 'Hex'
     */
    as?: as | 'Hex' | 'Bytes' | undefined
  }

  type ReturnType<as extends 'Hex' | 'Bytes'> = {
    privateKey:
      | (as extends 'Bytes' ? Bytes.Bytes : never)
      | (as extends 'Hex' ? Hex.Hex : never)
    publicKey:
      | (as extends 'Bytes' ? Bytes.Bytes : never)
      | (as extends 'Hex' ? Hex.Hex : never)
  }

  type ErrorType =
    | Hex.fromBytes.ErrorType
    | randomPrivateKey.ErrorType
    | getPublicKey.ErrorType
    | Errors.GlobalErrorType
}

/**
 * Computes the X25519 public key from a provided private key.
 *
 * @example
 * ```ts twoslash
 * import { X25519 } from 'ox'
 *
 * const publicKey = X25519.getPublicKey({ privateKey: '0x...' })
 * ```
 *
 * @param options - The options to compute the public key.
 * @returns The computed public key.
 */
export function getPublicKey<as extends 'Hex' | 'Bytes' = 'Hex'>(
  options: getPublicKey.Options<as>,
): getPublicKey.ReturnType<as> {
  const { as = 'Hex', privateKey } = options
  const privateKeyBytes = Bytes.from(privateKey)
  const publicKeyBytes = x25519.getPublicKey(privateKeyBytes)
  if (as === 'Hex') return Hex.fromBytes(publicKeyBytes) as never
  return publicKeyBytes as never
}

export declare namespace getPublicKey {
  type Options<as extends 'Hex' | 'Bytes' = 'Hex'> = {
    /**
     * Format of the returned public key.
     * @default 'Hex'
     */
    as?: as | 'Hex' | 'Bytes' | undefined
    /**
     * Private key to compute the public key from.
     */
    privateKey: Hex.Hex | Bytes.Bytes
  }

  type ReturnType<as extends 'Hex' | 'Bytes'> =
    | (as extends 'Bytes' ? Bytes.Bytes : never)
    | (as extends 'Hex' ? Hex.Hex : never)

  type ErrorType =
    | Bytes.from.ErrorType
    | Hex.fromBytes.ErrorType
    | Errors.GlobalErrorType
}

/**
 * Computes a shared secret using X25519 elliptic curve Diffie-Hellman between a private key and a public key.
 *
 * @example
 * ```ts twoslash
 * import { X25519 } from 'ox'
 *
 * const { privateKey: privateKeyA } = X25519.createKeyPair()
 * const { publicKey: publicKeyB } = X25519.createKeyPair()
 *
 * const sharedSecret = X25519.getSharedSecret({
 *   privateKey: privateKeyA,
 *   publicKey: publicKeyB
 * })
 * ```
 *
 * @param options - The options to compute the shared secret.
 * @returns The computed shared secret.
 */
export function getSharedSecret<as extends 'Hex' | 'Bytes' = 'Hex'>(
  options: getSharedSecret.Options<as>,
): getSharedSecret.ReturnType<as> {
  const { as = 'Hex', privateKey, publicKey } = options
  const privateKeyBytes = Bytes.from(privateKey)
  const publicKeyBytes = Bytes.from(publicKey)
  const sharedSecretBytes = x25519.getSharedSecret(
    privateKeyBytes,
    publicKeyBytes,
  )
  if (as === 'Hex') return Hex.fromBytes(sharedSecretBytes) as never
  return sharedSecretBytes as never
}

export declare namespace getSharedSecret {
  type Options<as extends 'Hex' | 'Bytes' = 'Hex'> = {
    /**
     * Format of the returned shared secret.
     * @default 'Hex'
     */
    as?: as | 'Hex' | 'Bytes' | undefined
    /**
     * Private key to use for the shared secret computation.
     */
    privateKey: Hex.Hex | Bytes.Bytes
    /**
     * Public key to use for the shared secret computation.
     */
    publicKey: Hex.Hex | Bytes.Bytes
  }

  type ReturnType<as extends 'Hex' | 'Bytes'> =
    | (as extends 'Bytes' ? Bytes.Bytes : never)
    | (as extends 'Hex' ? Hex.Hex : never)

  type ErrorType =
    | Bytes.from.ErrorType
    | Hex.fromBytes.ErrorType
    | Errors.GlobalErrorType
}

/**
 * Generates a random X25519 private key.
 *
 * @example
 * ```ts twoslash
 * import { X25519 } from 'ox'
 *
 * const privateKey = X25519.randomPrivateKey()
 * ```
 *
 * @param options - The options to generate the private key.
 * @returns The generated private key.
 */
export function randomPrivateKey<as extends 'Hex' | 'Bytes' = 'Hex'>(
  options: randomPrivateKey.Options<as> = {},
): randomPrivateKey.ReturnType<as> {
  const { as = 'Hex' } = options
  const bytes = x25519.utils.randomPrivateKey()
  if (as === 'Hex') return Hex.fromBytes(bytes) as never
  return bytes as never
}

export declare namespace randomPrivateKey {
  type Options<as extends 'Hex' | 'Bytes' = 'Hex'> = {
    /**
     * Format of the returned private key.
     * @default 'Hex'
     */
    as?: as | 'Hex' | 'Bytes' | undefined
  }

  type ReturnType<as extends 'Hex' | 'Bytes'> =
    | (as extends 'Bytes' ? Bytes.Bytes : never)
    | (as extends 'Hex' ? Hex.Hex : never)

  type ErrorType = Hex.fromBytes.ErrorType | Errors.GlobalErrorType
}
