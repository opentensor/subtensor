import * as Bytes from './Bytes.js';
import type * as Errors from './Errors.js';
import * as Hex from './Hex.js';
/** Re-export of noble/curves Ed25519 utilities. */
export declare const noble: import("@noble/curves/abstract/edwards").CurveFn;
/**
 * Creates a new Ed25519 key pair consisting of a private key and its corresponding public key.
 *
 * @example
 * ```ts twoslash
 * import { Ed25519 } from 'ox'
 *
 * const { privateKey, publicKey } = Ed25519.createKeyPair()
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
 * Computes the Ed25519 public key from a provided private key.
 *
 * @example
 * ```ts twoslash
 * import { Ed25519 } from 'ox'
 *
 * const publicKey = Ed25519.getPublicKey({ privateKey: '0x...' })
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
 * Generates a random Ed25519 private key.
 *
 * @example
 * ```ts twoslash
 * import { Ed25519 } from 'ox'
 *
 * const privateKey = Ed25519.randomPrivateKey()
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
/**
 * Signs the payload with the provided private key and returns an Ed25519 signature.
 *
 * @example
 * ```ts twoslash
 * import { Ed25519 } from 'ox'
 *
 * const signature = Ed25519.sign({ // [!code focus]
 *   payload: '0xdeadbeef', // [!code focus]
 *   privateKey: '0x...' // [!code focus]
 * }) // [!code focus]
 * ```
 *
 * @param options - The signing options.
 * @returns The Ed25519 signature.
 */
export declare function sign<as extends 'Hex' | 'Bytes' = 'Hex'>(options: sign.Options<as>): sign.ReturnType<as>;
export declare namespace sign {
    type Options<as extends 'Hex' | 'Bytes' = 'Hex'> = {
        /**
         * Format of the returned signature.
         * @default 'Hex'
         */
        as?: as | 'Hex' | 'Bytes' | undefined;
        /**
         * Payload to sign.
         */
        payload: Hex.Hex | Bytes.Bytes;
        /**
         * Ed25519 private key.
         */
        privateKey: Hex.Hex | Bytes.Bytes;
    };
    type ReturnType<as extends 'Hex' | 'Bytes'> = (as extends 'Bytes' ? Bytes.Bytes : never) | (as extends 'Hex' ? Hex.Hex : never);
    type ErrorType = Bytes.from.ErrorType | Hex.fromBytes.ErrorType | Errors.GlobalErrorType;
}
/**
 * Verifies a payload was signed by the provided public key.
 *
 * @example
 * ```ts twoslash
 * import { Ed25519 } from 'ox'
 *
 * const { privateKey, publicKey } = Ed25519.createKeyPair()
 * const signature = Ed25519.sign({ payload: '0xdeadbeef', privateKey })
 *
 * const verified = Ed25519.verify({ // [!code focus]
 *   publicKey, // [!code focus]
 *   payload: '0xdeadbeef', // [!code focus]
 *   signature, // [!code focus]
 * }) // [!code focus]
 * ```
 *
 * @param options - The verification options.
 * @returns Whether the payload was signed by the provided public key.
 */
export declare function verify(options: verify.Options): boolean;
export declare namespace verify {
    type Options = {
        /** Payload that was signed. */
        payload: Hex.Hex | Bytes.Bytes;
        /** Public key that signed the payload. */
        publicKey: Hex.Hex | Bytes.Bytes;
        /** Signature of the payload. */
        signature: Hex.Hex | Bytes.Bytes;
    };
    type ErrorType = Bytes.from.ErrorType | Errors.GlobalErrorType;
}
//# sourceMappingURL=Ed25519.d.ts.map