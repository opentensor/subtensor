import { stringToU8a, u8aFixLength } from '@polkadot/util';
import { naclDecrypt } from '../nacl/index.js';
import { scryptEncode, scryptFromU8a } from '../scrypt/index.js';
import { ENCODING, NONCE_LENGTH, SCRYPT_LENGTH } from './constants.js';
export function jsonDecryptData(encrypted, passphrase, encType = ENCODING) {
    if (!encrypted) {
        throw new Error('No encrypted data available to decode');
    }
    else if (encType.includes('xsalsa20-poly1305') && !passphrase) {
        throw new Error('Password required to decode encrypted data');
    }
    let encoded = encrypted;
    if (passphrase) {
        let password;
        if (encType.includes('scrypt')) {
            const { params, salt } = scryptFromU8a(encrypted);
            password = scryptEncode(passphrase, salt, params).password;
            encrypted = encrypted.subarray(SCRYPT_LENGTH);
        }
        else {
            password = stringToU8a(passphrase);
        }
        encoded = naclDecrypt(encrypted.subarray(NONCE_LENGTH), encrypted.subarray(0, NONCE_LENGTH), u8aFixLength(password, 256, true));
    }
    if (!encoded) {
        throw new Error('Unable to decode using the supplied passphrase');
    }
    return encoded;
}
