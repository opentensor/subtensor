import * as Bytes from './Bytes.js';
import type * as Errors from './Errors.js';
import * as Hex from './Hex.js';
/** Re-export of noble/curves X25519 utilities. */
export declare const noble: import("@noble/curves/abstract/montgomery").CurveFn;
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
export declare function createKeyPair<as extends 'Hex' | 'Bytes' = 'Hex'>(options?: createKeyPair.Options<as>): createKeyPair.ReturnType<as>;
export declare namespace createKeyPair {
    type Options<as extends 'Hex' | 'Bytes' = 'Hex'> = {
        /**
         * Format of the returned private and public keys.
         * @default 'Hex'
         */
        as?: as | 'Hex' | 'Bytes' | undefined;
    };
    type ReturnType<as extends 'Hex' | 'Bytes'> = {
        privateKey: (as extends 'Bytes' ? Bytes.Bytes : never) | (as extends 'Hex' ? Hex.Hex : never);
        publicKey: (as extends 'Bytes' ? Bytes.Bytes : never) | (as extends 'Hex' ? Hex.Hex : never);
    };
    type ErrorType = Hex.fromBytes.ErrorType | randomPrivateKey.ErrorType | getPublicKey.ErrorType | Errors.GlobalErrorType;
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
export declare function getPublicKey<as extends 'Hex' | 'Bytes' = 'Hex'>(options: getPublicKey.Options<as>): getPublicKey.ReturnType<as>;
export declare namespace getPublicKey {
    type Options<as extends 'Hex' | 'Bytes' = 'Hex'> = {
        /**
         * Format of the returned public key.
         * @default 'Hex'
         */
        as?: as | 'Hex' | 'Bytes' | undefined;
        /**
         * Private key to compute the public key from.
         */
        privateKey: Hex.Hex | Bytes.Bytes;
    };
    type ReturnType<as extends 'Hex' | 'Bytes'> = (as extends 'Bytes' ? Bytes.Bytes : never) | (as extends 'Hex' ? Hex.Hex : never);
    type ErrorType = Bytes.from.ErrorType | Hex.fromBytes.ErrorType | Errors.GlobalErrorType;
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
export declare function getSharedSecret<as extends 'Hex' | 'Bytes' = 'Hex'>(options: getSharedSecret.Options<as>): getSharedSecret.ReturnType<as>;
export declare namespace getSharedSecret {
    type Options<as extends 'Hex' | 'Bytes' = 'Hex'> = {
        /**
         * Format of the returned shared secret.
         * @default 'Hex'
         */
        as?: as | 'Hex' | 'Bytes' | undefined;
        /**
         * Private key to use for the shared secret computation.
         */
        privateKey: Hex.Hex | Bytes.Bytes;
        /**
         * Public key to use for the shared secret computation.
         */
        publicKey: Hex.Hex | Bytes.Bytes;
    };
    type ReturnType<as extends 'Hex' | 'Bytes'> = (as extends 'Bytes' ? Bytes.Bytes : never) | (as extends 'Hex' ? Hex.Hex : never);
    type ErrorType = Bytes.from.ErrorType | Hex.fromBytes.ErrorType | Errors.GlobalErrorType;
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
export declare function randomPrivateKey<as extends 'Hex' | 'Bytes' = 'Hex'>(options?: randomPrivateKey.Options<as>): randomPrivateKey.ReturnType<as>;
export declare namespace randomPrivateKey {
    type Options<as extends 'Hex' | 'Bytes' = 'Hex'> = {
        /**
         * Format of the returned private key.
         * @default 'Hex'
         */
        as?: as | 'Hex' | 'Bytes' | undefined;
    };
    type ReturnType<as extends 'Hex' | 'Bytes'> = (as extends 'Bytes' ? Bytes.Bytes : never) | (as extends 'Hex' ? Hex.Hex : never);
    type ErrorType = Hex.fromBytes.ErrorType | Errors.GlobalErrorType;
}
//# sourceMappingURL=X25519.d.ts.map