import { validateAddress } from './validate.js';
export function isAddress(address, ignoreChecksum, ss58Format) {
    try {
        return validateAddress(address, ignoreChecksum, ss58Format);
    }
    catch {
        return false;
    }
}
