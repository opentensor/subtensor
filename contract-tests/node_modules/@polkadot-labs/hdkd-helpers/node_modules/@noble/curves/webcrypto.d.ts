/**
 * Friendly wrapper over elliptic curves from built-in WebCrypto. Experimental: API may change.

# WebCrypto issues

## No way to get public keys

- Export of raw secret key is prohibited by spec:
  - https://w3c.github.io/webcrypto/#ecdsa-operations-export-key
    -> "If format is "raw":" -> "If the [[type]] internal slot of key is not "public",
       then throw an InvalidAccessError."
- Import of raw secret keys is prohibited by spec:
  - https://w3c.github.io/webcrypto/#ecdsa-operations-import-key
    -> "If format is "raw":" -> "If usages contains a value which is not "verify"
       then throw a SyntaxError."
- SPKI (Simple public-key infrastructure) is public-key-only
- PKCS8 is secret-key-only
- No way to get public key from secret key, but we convert to jwk and then create it manually, since jwk secret key is priv+pub.
- Noble supports generating keys for both sign, verify & getSharedSecret,
  but JWK key includes usage, which forces us to patch it (non-JWK is ok)
- We have import/export for 'raw', but it doesn't work in Firefox / Safari

## Point encoding

- Raw export of public points returns uncompressed points,
  but this is implementation specific and not much we can do there.
- `getSharedSecret` differs for p256, p384, p521:
  Noble returns 33-byte output (y-parity + x coordinate),
  while in WebCrypto returns 32-byte output (x coordinate)
- `getSharedSecret` identical for X25519, X448

## Availability

Node.js additionally supports ed448.
There seems no reasonable way to check for availability, other than actually calling methods.

 * @module
 */
/*! noble-curves - MIT License (c) 2022 Paul Miller (paulmillr.com) */
/** Raw type */
declare const TYPE_RAW = "raw";
declare const TYPE_JWK = "jwk";
declare const TYPE_SPKI = "spki";
declare const TYPE_PKCS = "pkcs8";
export type WebCryptoFormat = typeof TYPE_RAW | typeof TYPE_JWK | typeof TYPE_SPKI | typeof TYPE_PKCS;
/** WebCrypto keys can be in raw, jwk, pkcs8/spki formats. Raw is internal and fragile. */
export type WebCryptoOpts = {
    formatSec?: WebCryptoFormat;
    formatPub?: WebCryptoFormat;
};
type JsonWebKey = {
    crv?: string;
    d?: string;
    kty?: string;
    x?: string;
    y?: string;
    [key: string]: unknown;
};
type Key = JsonWebKey | Uint8Array;
type WebCryptoBaseCurve = {
    name: string;
    isSupported(): Promise<boolean>;
    keygen(): Promise<{
        secretKey: Uint8Array;
        publicKey: Uint8Array;
    }>;
    getPublicKey(secretKey: Key, opts?: WebCryptoOpts): Promise<Key>;
    utils: {
        randomSecretKey: (format?: WebCryptoFormat) => Promise<Key>;
        convertSecretKey: (key: Key, inFormat?: WebCryptoFormat, outFormat?: WebCryptoFormat) => Promise<Key>;
        convertPublicKey: (key: Key, inFormat?: WebCryptoFormat, outFormat?: WebCryptoFormat) => Promise<Key>;
    };
};
export type WebCryptoSigner = {
    sign(message: Uint8Array, secretKey: Key, opts?: WebCryptoOpts): Promise<Uint8Array>;
    verify(signature: Uint8Array, message: Uint8Array, publicKey: Key, opts?: WebCryptoOpts): Promise<boolean>;
};
export type WebCryptoECDH = {
    getSharedSecret(secA: Uint8Array, pubB: Uint8Array, opts?: WebCryptoOpts): Promise<Uint8Array>;
};
export type WebCryptoECDSA = WebCryptoBaseCurve & WebCryptoSigner & WebCryptoECDH;
export type WebCryptoEdDSA = WebCryptoBaseCurve & WebCryptoSigner;
export type WebCryptoMontgomery = WebCryptoBaseCurve & WebCryptoECDH;
/** Friendly wrapper over built-in WebCrypto NIST P-256 (secp256r1). */
export declare const p256: WebCryptoECDSA;
/** Friendly wrapper over built-in WebCrypto NIST P-384 (secp384r1). */
export declare const p384: WebCryptoECDSA;
/** Friendly wrapper over built-in WebCrypto NIST P-521 (secp521r1). */
export declare const p521: WebCryptoECDSA;
/** Friendly wrapper over built-in WebCrypto ed25519. */
export declare const ed25519: WebCryptoEdDSA;
/** Friendly wrapper over built-in WebCrypto ed448. */
export declare const ed448: WebCryptoEdDSA;
/** Friendly wrapper over built-in WebCrypto x25519 (ECDH over Curve25519). */
export declare const x25519: WebCryptoMontgomery;
/** Friendly wrapper over built-in WebCrypto x448 (ECDH over Curve448). */
export declare const x448: WebCryptoMontgomery;
export {};
//# sourceMappingURL=webcrypto.d.ts.map