import { universalResolverReverseAbi } from '../../constants/abis.js';
import { getChainContractAddress, } from '../../utils/chain/getChainContractAddress.js';
import { isNullUniversalResolverError } from '../../utils/ens/errors.js';
import { localBatchGatewayUrl } from '../../utils/ens/localBatchGatewayRequest.js';
import { getAction } from '../../utils/getAction.js';
import { readContract, } from '../public/readContract.js';
/**
 * Gets primary name for specified address.
 *
 * - Docs: https://viem.sh/docs/ens/actions/getEnsName
 * - Examples: https://stackblitz.com/github/wevm/viem/tree/main/examples/ens
 *
 * Calls `reverse(bytes)` on ENS Universal Resolver Contract to "reverse resolve" the address to the primary ENS name.
 *
 * @param client - Client to use
 * @param parameters - {@link GetEnsNameParameters}
 * @returns Name or `null` if not found. {@link GetEnsNameReturnType}
 *
 * @example
 * import { createPublicClient, http } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { getEnsName } from 'viem/ens'
 *
 * const client = createPublicClient({
 *   chain: mainnet,
 *   transport: http(),
 * })
 * const ensName = await getEnsName(client, {
 *   address: '0xd2135CfB216b74109775236E36d4b433F1DF507B',
 * })
 * // 'wevm.eth'
 */
export async function getEnsName(client, parameters) {
    const { address, blockNumber, blockTag, coinType = 60n, gatewayUrls, strict, } = parameters;
    const { chain } = client;
    const universalResolverAddress = (() => {
        if (parameters.universalResolverAddress)
            return parameters.universalResolverAddress;
        if (!chain)
            throw new Error('client chain not configured. universalResolverAddress is required.');
        return getChainContractAddress({
            blockNumber,
            chain,
            contract: 'ensUniversalResolver',
        });
    })();
    try {
        const readContractParameters = {
            address: universalResolverAddress,
            abi: universalResolverReverseAbi,
            args: [address, coinType, gatewayUrls ?? [localBatchGatewayUrl]],
            functionName: 'reverseWithGateways',
            blockNumber,
            blockTag,
        };
        const readContractAction = getAction(client, readContract, 'readContract');
        const [name] = await readContractAction(readContractParameters);
        return name || null;
    }
    catch (err) {
        if (strict)
            throw err;
        if (isNullUniversalResolverError(err))
            return null;
        throw err;
    }
}
//# sourceMappingURL=getEnsName.js.map