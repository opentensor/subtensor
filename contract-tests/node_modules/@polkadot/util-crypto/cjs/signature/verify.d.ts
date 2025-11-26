import type { VerifyResult } from '../types.js';
export declare function signatureVerify(message: string | Uint8Array, signature: string | Uint8Array, addressOrPublicKey: string | Uint8Array): VerifyResult;
