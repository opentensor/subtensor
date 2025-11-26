import { SignatureErc8010 } from 'ox/erc8010';
import { hexToBytes } from '../encoding/toBytes.js';
/**
 * @description Serializes a ERC-8010 flavoured signature into hex format.
 *
 * @param signature ERC-8010 signature in object format.
 * @returns ERC-8010 signature in hex format.
 */
export function serializeErc8010Signature(parameters) {
    const { address, data, signature, to = 'hex' } = parameters;
    const signature_ = SignatureErc8010.wrap({
        authorization: {
            address: parameters.authorization.address,
            chainId: parameters.authorization.chainId,
            nonce: BigInt(parameters.authorization.nonce),
            r: BigInt(parameters.authorization.r),
            s: BigInt(parameters.authorization.s),
            yParity: parameters.authorization.yParity,
        },
        data,
        signature,
        to: address,
    });
    if (to === 'hex')
        return signature_;
    return hexToBytes(signature_);
}
//# sourceMappingURL=serializeErc8010Signature.js.map