import * as Bytes from './Bytes.js';
import type * as Errors from './Errors.js';
import * as Hex from './Hex.js';
import * as PublicKey from './PublicKey.js';
import type * as Signature from './Signature.js';
/** Re-export of noble/curves P256 utilities. */
export declare const noble: Readonly<{
    create: (hash: import("@noble/curves/abstract/utils").CHash) => import("@noble/curves/abstract/weierstrass").CurveFn;
    CURVE: ReturnType<(curve: import("@noble/curves/abstract/weierstrass").CurveType) => Readonly<{
        readonly nBitLength: number;
        readonly nByteLength: number;
        readonly Fp: import("@noble/curves/abstract/modular").IField<bigint>;
        readonly n: bigint;
        readonly h: bigint;
        readonly hEff?: bigint;
        readonly Gx: bigint;
        readonly Gy: bigint;
        readonly allowInfinityPoint?: boolean;
        readonly a: bigint;
        readonly b: bigint;
        readonly allowedPrivateKeyLengths?: readonly number[];
        readonly wrapPrivateKey?: boolean;
        readonly endo?: {
            beta: bigint;
            splitScalar: (k: bigint) => {
                k1neg: boolean;
                k1: bigint;
                k2neg: boolean;
                k2: bigint;
            };
        };
        readonly isTorsionFree?: ((c: import("@noble/curves/abstract/weierstrass").ProjConstructor<bigint>, point: import("@noble/curves/abstract/weierstrass").ProjPointType<bigint>) => boolean) | undefined;
        readonly clearCofactor?: ((c: import("@noble/curves/abstract/weierstrass").ProjConstructor<bigint>, point: import("@noble/curves/abstract/weierstrass").ProjPointType<bigint>) => import("@noble/curves/abstract/weierstrass").ProjPointType<bigint>) | undefined;
        readonly hash: import("@noble/curves/abstract/utils").CHash;
        readonly hmac: (key: Uint8Array, ...messages: Uint8Array[]) => Uint8Array;
        readonly randomBytes: (bytesLength?: number) => Uint8Array;
        lowS: boolean;
        readonly bits2int?: (bytes: Uint8Array) => bigint;
        readonly bits2int_modN?: (bytes: Uint8Array) => bigint;
        readonly p: bigint;
    }>>;
    getPublicKey: (privateKey: import("@noble/curves/abstract/utils").PrivKey, isCompressed?: boolean) => Uint8Array;
    getSharedSecret: (privateA: import("@noble/curves/abstract/utils").PrivKey, publicB: import("@noble/curves/abstract/utils").Hex, isCompressed?: boolean) => Uint8Array;
    sign: (msgHash: import("@noble/curves/abstract/utils").Hex, privKey: import("@noble/curves/abstract/utils").PrivKey, opts?: import("@noble/curves/abstract/weierstrass").SignOpts) => import("@noble/curves/abstract/weierstrass").RecoveredSignatureType;
    verify: (signature: import("@noble/curves/abstract/utils").Hex | {
        r: bigint;
        s: bigint;
    }, msgHash: import("@noble/curves/abstract/utils").Hex, publicKey: import("@noble/curves/abstract/utils").Hex, opts?: import("@noble/curves/abstract/weierstrass").VerOpts) => boolean;
    ProjectivePoint: import("@noble/curves/abstract/weierstrass").ProjConstructor<bigint>;
    Signature: import("@noble/curves/abstract/weierstrass").SignatureConstructor;
    utils: {
        normPrivateKeyToScalar: (key: import("@noble/curves/abstract/utils").PrivKey) => bigint;
        isValidPrivateKey(privateKey: import("@noble/curves/abstract/utils").PrivKey): boolean;
        randomPrivateKey: () => Uint8Array;
        precompute: (windowSize?: number, point?: import("@noble/curves/abstract/weierstrass").ProjPointType<bigint>) => import("@noble/curves/abstract/weierstrass").ProjPointType<bigint>;
    };
}>;
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
 * const privateKey = P256.randomPrivateKey()
 * const publicKey = P256.getPublicKey({ privateKey })
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