import { hexToU8a, isHex } from '@polkadot/util';
import { base64Decode } from '../base64/index.js';
import { jsonDecryptData } from './decryptData.js';
export function jsonDecrypt({ encoded, encoding }, passphrase) {
    if (!encoded) {
        throw new Error('No encrypted data available to decode');
    }
    return jsonDecryptData(isHex(encoded)
        ? hexToU8a(encoded)
        : base64Decode(encoded), passphrase, Array.isArray(encoding.type)
        ? encoding.type
        : [encoding.type]);
}
