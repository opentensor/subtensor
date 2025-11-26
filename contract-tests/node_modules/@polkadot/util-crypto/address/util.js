import { decodeAddress } from './decode.js';
export function addressToU8a(who) {
    return decodeAddress(who);
}
