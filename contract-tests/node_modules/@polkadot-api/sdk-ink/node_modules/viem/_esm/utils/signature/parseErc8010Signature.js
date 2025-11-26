import { SignatureErc8010 } from 'ox/erc8010';
import { numberToHex } from '../encoding/toHex.js';
import { isErc8010Signature, } from './isErc8010Signature.js';
/**
 * @description Parses a hex-formatted ERC-8010 flavoured signature.
 * If the signature is not in ERC-8010 format, then the underlying (original) signature is returned.
 *
 * @param signature ERC-8010 signature in hex format.
 * @returns The parsed ERC-8010 signature.
 */
export function parseErc8010Signature(signature) {
    if (!isErc8010Signature(signature))
        return { signature };
    const { authorization: authorization_ox, to, ...rest } = SignatureErc8010.unwrap(signature);
    return {
        authorization: {
            address: authorization_ox.address,
            chainId: authorization_ox.chainId,
            nonce: Number(authorization_ox.nonce),
            r: numberToHex(authorization_ox.r, { size: 32 }),
            s: numberToHex(authorization_ox.s, { size: 32 }),
            yParity: authorization_ox.yParity,
        },
        ...(to ? { address: to } : {}),
        ...rest,
    };
}
//# sourceMappingURL=parseErc8010Signature.js.map