/**
 * @name sr25519Agreement
 * @description Key agreement between other's public key and self secret key
 */
export declare function sr25519Agreement(secretKey: string | Uint8Array, publicKey: string | Uint8Array): Uint8Array;
