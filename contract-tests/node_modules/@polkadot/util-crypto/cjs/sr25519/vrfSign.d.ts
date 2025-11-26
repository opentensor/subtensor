import type { Keypair } from '../types.js';
/**
 * @name sr25519VrfSign
 * @description Sign with sr25519 vrf signing (deterministic)
 */
export declare function sr25519VrfSign(message: string | Uint8Array, { secretKey }: Partial<Keypair>, context?: string | Uint8Array, extra?: string | Uint8Array): Uint8Array;
