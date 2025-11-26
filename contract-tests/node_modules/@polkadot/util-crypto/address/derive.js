import { keyExtractPath } from '../key/index.js';
import { sr25519DerivePublic } from '../sr25519/index.js';
import { decodeAddress } from './decode.js';
import { encodeAddress } from './encode.js';
function filterHard({ isHard }) {
    return isHard;
}
/**
 * @name deriveAddress
 * @summary Creates a sr25519 derived address from the supplied and path.
 * @description
 * Creates a sr25519 derived address based on the input address/publicKey and the uri supplied.
 */
export function deriveAddress(who, suri, ss58Format) {
    const { path } = keyExtractPath(suri);
    if (!path.length || path.every(filterHard)) {
        throw new Error('Expected suri to contain a combination of non-hard paths');
    }
    let publicKey = decodeAddress(who);
    for (const { chainCode } of path) {
        publicKey = sr25519DerivePublic(publicKey, chainCode);
    }
    return encodeAddress(publicKey, ss58Format);
}
