import { addressResolverAbi, universalResolverResolveAbi, } from '../../constants/abis.js';
import { decodeFunctionResult, } from '../../utils/abi/decodeFunctionResult.js';
import { encodeFunctionData, } from '../../utils/abi/encodeFunctionData.js';
import { getChainContractAddress, } from '../../utils/chain/getChainContractAddress.js';
import { trim } from '../../utils/data/trim.js';
import { toHex } from '../../utils/encoding/toHex.js';
import { isNullUniversalResolverError } from '../../utils/ens/errors.js';
import { localBatchGatewayUrl } from '../../utils/ens/localBatchGatewayRequest.js';
import { namehash } from '../../utils/ens/namehash.js';
import { packetToBytes, } from '../../utils/ens/packetToBytes.js';
import { getAction } from '../../utils/getAction.js';
import { readContract, } from '../public/readContract.js';
/**
 * Gets address for ENS name.
 *
 * - Docs: https://viem.sh/docs/ens/actions/getEnsAddress
 * - Examples: https://stackblitz.com/github/wevm/viem/tree/main/examples/ens
 *
 * Calls `resolve(bytes, bytes)` on ENS Universal Resolver Contract.
 *
 * Since ENS names prohibit certain forbidden characters (e.g. underscore) and have other validation rules, you likely want to [normalize ENS names](https://docs.ens.domains/contract-api-reference/name-processing#normalising-names) with [UTS-46 normalization](https://unicode.org/reports/tr46) before passing them to `getEnsAddress`. You can use the built-in [`normalize`](https://viem.sh/docs/ens/utilities/normalize) function for this.
 *
 * @param client - Client to use
 * @param parameters - {@link GetEnsAddressParameters}
 * @returns Address for ENS name or `null` if not found. {@link GetEnsAddressReturnType}
 *
 * @example
 * import { createPublicClient, http } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { getEnsAddress, normalize } from 'viem/ens'
 *
 * const client = createPublicClient({
 *   chain: mainnet,
 *   transport: http(),
 * })
 * const ensAddress = await getEnsAddress(client, {
 *   name: normalize('wevm.eth'),
 * })
 * // '0xd2135CfB216b74109775236E36d4b433F1DF507B'
 */
export async function getEnsAddress(client, parameters) {
    const { blockNumber, blockTag, coinType, name, gatewayUrls, strict } = parameters;
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
    const tlds = chain?.ensTlds;
    if (tlds && !tlds.some((tld) => name.endsWith(tld)))
        return null;
    const args = (() => {
        if (coinType != null)
            return [namehash(name), BigInt(coinType)];
        return [namehash(name)];
    })();
    try {
        const functionData = encodeFunctionData({
            abi: addressResolverAbi,
            functionName: 'addr',
            args,
        });
        const readContractParameters = {
            address: universalResolverAddress,
            abi: universalResolverResolveAbi,
            functionName: 'resolveWithGateways',
            args: [
                toHex(packetToBytes(name)),
                functionData,
                gatewayUrls ?? [localBatchGatewayUrl],
            ],
            blockNumber,
            blockTag,
        };
        const readContractAction = getAction(client, readContract, 'readContract');
        const res = await readContractAction(readContractParameters);
        if (res[0] === '0x')
            return null;
        const address = decodeFunctionResult({
            abi: addressResolverAbi,
            args,
            functionName: 'addr',
            data: res[0],
        });
        if (address === '0x')
            return null;
        if (trim(address) === '0x00')
            return null;
        return address;
    }
    catch (err) {
        if (strict)
            throw err;
        if (isNullUniversalResolverError(err))
            return null;
        throw err;
    }
}
//# sourceMappingURL=getEnsAddress.js.map