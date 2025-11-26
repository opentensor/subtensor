import type { EncryptedJsonEncoding } from './types.js';
export declare function jsonDecryptData(encrypted?: Uint8Array | null, passphrase?: string | null, encType?: EncryptedJsonEncoding[]): Uint8Array;
