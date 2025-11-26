import { isHex, isU8a, u8aToU8a } from '@polkadot/util';
import { base58Decode } from '../base58/index.js';
import { checkAddressChecksum } from './checksum.js';
import { defaults } from './defaults.js';
export function decodeAddress(encoded, ignoreChecksum, ss58Format = -1) {
    if (!encoded) {
        throw new Error('Invalid empty address passed');
    }
    if (isU8a(encoded) || isHex(encoded)) {
        return u8aToU8a(encoded);
    }
    try {
        const decoded = base58Decode(encoded);
        if (!defaults.allowedEncodedLengths.includes(decoded.length)) {
            throw new Error('Invalid decoded address length');
        }
        const [isValid, endPos, ss58Length, ss58Decoded] = checkAddressChecksum(decoded);
        if (!isValid && !ignoreChecksum) {
            throw new Error('Invalid decoded address checksum');
        }
        else if (ss58Format !== -1 && ss58Format !== ss58Decoded) {
            throw new Error(`Expected ss58Format ${ss58Format}, received ${ss58Decoded}`);
        }
        return decoded.slice(ss58Length, endPos);
    }
    catch (error) {
        throw new Error(`Decoding ${encoded}: ${error.message}`);
    }
}
