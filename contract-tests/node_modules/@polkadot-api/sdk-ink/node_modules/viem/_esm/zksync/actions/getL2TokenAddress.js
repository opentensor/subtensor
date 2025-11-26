import { readContract } from '../../actions/public/readContract.js';
import { isAddressEqual } from '../../utils/index.js';
import { l2SharedBridgeAbi } from '../constants/abis.js';
import { ethAddressInContracts, l2BaseTokenAddress, legacyEthAddress, } from '../constants/address.js';
import { getBaseTokenL1Address } from './getBaseTokenL1Address.js';
import { getDefaultBridgeAddresses } from './getDefaultBridgeAddresses.js';
/**
 * Returns the L2 token address equivalent for a L1 token address as they are not equal.
 * ETH address is set to zero address.
 *
 * @remarks Only works for tokens bridged on default ZKsync Era bridges.
 *
 * @param client - Client to use
 * @param parameters - {@link GetL2TokenAddressParameters}
 * @returns The L2 token address equivalent for a L1 token address.
 *
 *
 * @example
 * import { createPublicClient, http } from 'viem'
 * import { zksync } from 'viem/chains'
 * import { publicActionsL2 } from 'viem/zksync'
 *
 * const client = createPublicClient({
 *   chain: zksync,
 *   transport: http(),
 * }).extend(publicActionsL2())
 *
 * const address = await getL2TokenAddress(client, {token: '0x...'});
 */
export async function getL2TokenAddress(client, parameters) {
    let { token, bridgeAddress } = parameters;
    if (isAddressEqual(token, legacyEthAddress))
        token = ethAddressInContracts;
    const baseToken = await getBaseTokenL1Address(client);
    if (isAddressEqual(token, baseToken))
        return l2BaseTokenAddress;
    bridgeAddress ??= (await getDefaultBridgeAddresses(client)).sharedL2;
    return await readContract(client, {
        address: bridgeAddress,
        abi: l2SharedBridgeAbi,
        functionName: 'l2TokenAddress',
        args: [token],
    });
}
//# sourceMappingURL=getL2TokenAddress.js.map