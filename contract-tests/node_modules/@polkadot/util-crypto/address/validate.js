import { decodeAddress } from './decode.js';
export function validateAddress(encoded, ignoreChecksum, ss58Format) {
    return !!decodeAddress(encoded, ignoreChecksum, ss58Format);
}
