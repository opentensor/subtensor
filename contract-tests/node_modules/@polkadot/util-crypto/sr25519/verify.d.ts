/**
 * @name sr25519Verify
 * @description Verifies the signature of `message`, using the supplied pair
 */
export declare function sr25519Verify(message: string | Uint8Array, signature: string | Uint8Array, publicKey: string | Uint8Array): boolean;
