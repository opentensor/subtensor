import type { EncryptedJson } from './types.js';
export declare function jsonDecrypt({ encoded, encoding }: EncryptedJson, passphrase?: string | null): Uint8Array;
