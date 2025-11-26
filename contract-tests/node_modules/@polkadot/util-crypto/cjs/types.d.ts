export * from './address/types.js';
export * from './json/types.js';
export interface Keypair {
    /** The publicKey for this pair */
    publicKey: Uint8Array;
    /** The secretKey for this pair */
    secretKey: Uint8Array;
}
export interface Seedpair {
    /** The publicKey for this pair */
    publicKey: Uint8Array;
    /** The seed used to construct the pair */
    seed: Uint8Array;
}
/** The supported types of pairs */
export type KeypairType = 'ed25519' | 'sr25519' | 'ecdsa' | 'ethereum';
export interface VerifyResult {
    /** The detected crypto interface, or 'none' if not detected */
    crypto: 'none' | KeypairType;
    /** The validity for this result, false if invalid */
    isValid: boolean;
    /** Flag to indicate if the passed data was wrapped in <Bytes>...</Bytes> */
    isWrapped: boolean;
    /** The extracted publicKey */
    publicKey: Uint8Array;
}
