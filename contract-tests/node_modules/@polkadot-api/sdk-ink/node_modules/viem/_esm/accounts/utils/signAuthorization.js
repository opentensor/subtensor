import { hashAuthorization, } from '../../utils/authorization/hashAuthorization.js';
import { sign, } from './sign.js';
/**
 * Signs an Authorization hash in [EIP-7702 format](https://eips.ethereum.org/EIPS/eip-7702): `keccak256('0x05' || rlp([chain_id, address, nonce]))`.
 */
export async function signAuthorization(parameters) {
    const { chainId, nonce, privateKey, to = 'object' } = parameters;
    const address = parameters.contractAddress ?? parameters.address;
    const signature = await sign({
        hash: hashAuthorization({ address, chainId, nonce }),
        privateKey,
        to,
    });
    if (to === 'object')
        return {
            address,
            chainId,
            nonce,
            ...signature,
        };
    return signature;
}
//# sourceMappingURL=signAuthorization.js.map