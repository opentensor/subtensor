import * as Bytes from './Bytes.js';
import type * as Errors from './Errors.js';
import * as Hex from './Hex.js';
import * as PublicKey from './PublicKey.js';
import type * as Signature from './Signature.js';
/** Re-export of noble/curves P256 utilities. */
export declare const noble: import("@noble/curves/_shortw_utils").CurveFnWithCreate;
/**
 * Creates a new P256 ECDSA key pair consisting of a private key and its corresponding public key.
 *
 * @example
 * ```ts twoslash
 * import { P256 } from 'ox'
 *
 * const { privateKey, publicKey } = P256.createKeyPair()
 * ```
 *
 * @param options - The options to generate the key pair.
 * @returns The generated key pair containing both private and public keys.
 */
export declare function createKeyPair<as extends 'Hex' | 'Bytes' = 'Hex'>(options?: createKeyPair.Options<as>): createKeyPair.ReturnType<as>;
export declare namespace createKeyPair {
    type Options<as extends 'Hex' | 'Bytes' = 'Hex'> = {
        /**
         * Format of the returned private key.
         * @default 'Hex'
         */
        as?: as | 'Hex' | 'Bytes' | undefined;
    };
    type ReturnType<as extends 'Hex' | 'Bytes'> = {
        privateKey: (as extends 'Bytes' ? Bytes.Bytes : never) | (as extends 'Hex' ? Hex.Hex : never);
        publicKey: PublicKey.PublicKey;
    };
    type ErrorType = Hex.fromBytes.ErrorType | PublicKey.from.ErrorType | Errors.GlobalErrorType;
}
/**
 * Computes the P256 ECDSA public key from a provided private key.
 *
 * @example
 * ```ts twoslash
 * import { P256 } from 'ox'
 *
 * const publicKey = P256.getPublicKey({ privateKey: '0x...' })
 * ```
 *
 * @param options - The options to compute the public key.
 * @returns The computed public key.
 */
export declare function getPublicKey(options: getPublicKey.Options): PublicKey.PublicKey;
export declare namespace getPublicKey {
    type Options = {
        /**
         * Private key to compute the public key from.
         */
        privateKey: Hex.Hex | Bytes.Bytes;
    };
    type ErrorType = Errors.GlobalErrorType;
}
/**
 * Computes a shared secret using ECDH (Elliptic Curve Diffie-Hellman) between a private key and a public key.
 *
 * @example
 * ```ts twoslash
 * import { P256 } from 'ox'
 *
 * const { privateKey: privateKeyA } = P256.createKeyPair()
 * const { publicKey: publicKeyB } = P256.createKeyPair()
 *
 * const sharedSecret = P256.getSharedSecret({
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
        publicKey: PublicKey.PublicKey<boolean>;
    };
    type ReturnType<as extends 'Hex' | 'Bytes'> = (as extends 'Bytes' ? Bytes.Bytes : never) | (as extends 'Hex' ? Hex.Hex : never);
    type ErrorType = Hex.fromBytes.ErrorType | PublicKey.toHex.ErrorType | Errors.GlobalErrorType;
}
/**
 * Generates a random P256 ECDSA private key.
 *
 * @example
 * ```ts twoslash
 * import { P256 } from 'ox'
 *
 * const privateKey = P256.randomPrivateKey()
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
 * Recovers the signing public key from the signed payload and signature.
 *
 * @example
 * ```ts twoslash
 * import { P256 } from 'ox'
 *
 * const signature = P256.sign({ payload: '0xdeadbeef', privateKey: '0x...' })
 *
 * const publicKey = P256.recoverPublicKey({ // [!code focus]
 *   payload: '0xdeadbeef', // [!code focus]
 *   signature, // [!code focus]
 * }) // [!code focus]
 * ```
 *
 * @param options - The recovery options.
 * @returns The recovered public key.
 */
export declare function recoverPublicKey(options: recoverPublicKey.Options): PublicKey.PublicKey;
export declare namespace recoverPublicKey {
    type Options = {
        /** Payload that was signed. */
        payload: Hex.Hex | Bytes.Bytes;
        /** Signature of the payload. */
        signature: Signature.Signature;
    };
    type ErrorType = PublicKey.from.ErrorType | Hex.fromBytes.ErrorType | Errors.GlobalErrorType;
}
/**
 * Signs the payload with the provided private key and returns a P256 signature.
 *
 * @example
 * ```ts twoslash
 * import { P256 } from 'ox'
 *
 * const signature = P256.sign({ // [!code focus]
 *   payload: '0xdeadbeef', // [!code focus]
 *   privateKey: '0x...' // [!code focus]
 * }) // [!code focus]
 * ```
 *
 * @param options - The signing options.
 * @returns The ECDSA {@link ox#Signature.Signature}.
 */
export declare function sign(options: sign.Options): Signature.Signature;
export declare namespace sign {
    type Options = {
        /**
         * Extra entropy to add to the signing process. Setting to `false` will disable it.
         * @default true
         */
        extraEntropy?: boolean | Hex.Hex | Bytes.Bytes | undefined;
        /**
         * If set to `true`, the payload will be hashed (sha256) before being signed.
         */
        hash?: boolean | undefined;
        /**
         * Payload to sign.
         */
        payload: Hex.Hex | Bytes.Bytes;
        /**
         * ECDSA private key.
         */
        privateKey: Hex.Hex | Bytes.Bytes;
    };
    type ErrorType = Bytes.fromHex.ErrorType | Errors.GlobalErrorType;
}
/**
 * Verifies a payload was signed by the provided public key.
 *
 * @example
 *
 * ```ts twoslash
 * import { P256 } from 'ox'
 *
 * const { privateKey, publicKey } = P256.createKeyPair()
 * const signature = P256.sign({ payload: '0xdeadbeef', privateKey })
 *
 * const verified = P256.verify({ // [!code focus]
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
        /** If set to `true`, the payload will be hashed (sha256) before being verified. */
        hash?: boolean | undefined;
        /** Payload that was signed. */
        payload: Hex.Hex | Bytes.Bytes;
        /** Public key that signed the payload. */
        publicKey: PublicKey.PublicKey<boolean>;
        /** Signature of the payload. */
        signature: Signature.Signature<boolean>;
    };
    type ErrorType = Errors.GlobalErrorType;
}
//# sourceMappingURL=P256.d.ts.map