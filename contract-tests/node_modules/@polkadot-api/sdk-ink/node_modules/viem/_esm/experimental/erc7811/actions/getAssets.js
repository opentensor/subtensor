import { parseAccount, } from '../../../accounts/utils/parseAccount.js';
import { ethAddress } from '../../../constants/address.js';
import { AccountNotFoundError } from '../../../errors/account.js';
import { hexToBigInt, } from '../../../utils/encoding/fromHex.js';
import { numberToHex, } from '../../../utils/encoding/toHex.js';
/**
 * Retrieves assets for a given account from the target Wallet.
 *
 * @example
 * import { createWalletClient, custom } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { getAssets } from 'viem/experimental'
 *
 * const client = createWalletClient({
 *   chain: mainnet,
 *   transport: custom(window.ethereum),
 * })
 *
 * const result = await getAssets(client, {
 *   account: '0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef',
 * })
 *
 * @param client - Client to use to make the request.
 * @param parameters - Parameters.
 * @returns Assets for the given account.
 */
export async function getAssets(client, ...[parameters]) {
    const { account = client.account, aggregate = true } = parameters ?? {};
    const result = await client.request({
        method: 'wallet_getAssets',
        params: [formatRequest({ ...parameters, account })],
    });
    const response = formatResponse(result);
    const aggregated = (() => {
        if (!aggregate)
            return undefined;
        const aggregated = {};
        for (const [chainId, assets] of Object.entries(response)) {
            if (chainId === '0')
                continue;
            const seen = new Set();
            for (const asset of assets) {
                const key = typeof aggregate === 'function'
                    ? aggregate(asset)
                    : (asset.address ?? ethAddress);
                const item = (aggregated[key] ?? {});
                if (seen.has(key))
                    continue;
                seen.add(key);
                aggregated[key] = {
                    ...asset,
                    balance: asset.balance + (item?.balance ?? 0n),
                    chainIds: [...(item?.chainIds ?? []), Number(chainId)],
                };
            }
        }
        return Object.values(aggregated);
    })();
    if (aggregated)
        return { 0: aggregated, ...response };
    return response;
}
/** @internal */
function formatRequest(parameters = {}) {
    const { account: account_, assets, assetTypes, chainIds } = parameters;
    if (typeof account_ === 'undefined')
        throw new AccountNotFoundError({
            docsPath: '/experimental/erc7811/getAssets',
        });
    const account = parseAccount(account_);
    return {
        account: account.address,
        assetFilter: assets,
        assetTypeFilter: assetTypes,
        chainFilter: chainIds?.map((chainId) => numberToHex(chainId)),
    };
}
/** @internal */
function formatResponse(response) {
    return Object.fromEntries(Object.entries(response).map(([chainId, assets]) => [
        Number(chainId),
        assets.map((asset) => {
            const balance = hexToBigInt(asset.balance);
            const metadata = asset.metadata;
            const type = (() => {
                if (asset.type === 'native')
                    return 'native';
                if (asset.type === 'erc20')
                    return 'erc20';
                if (asset.type === 'erc721')
                    return 'erc721';
                return { custom: asset.type };
            })();
            const address = type === 'native' ? undefined : asset.address;
            return {
                balance,
                type,
                ...(address ? { address } : {}),
                ...(metadata
                    ? {
                        metadata: {
                            ...metadata,
                            ...('tokenId' in metadata
                                ? { tokenId: hexToBigInt(metadata.tokenId) }
                                : {}),
                        },
                    }
                    : {}),
            };
        }),
    ]));
}
//# sourceMappingURL=getAssets.js.map