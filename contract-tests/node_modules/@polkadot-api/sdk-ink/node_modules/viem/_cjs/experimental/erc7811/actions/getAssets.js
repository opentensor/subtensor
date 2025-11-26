"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getAssets = getAssets;
const parseAccount_js_1 = require("../../../accounts/utils/parseAccount.js");
const address_js_1 = require("../../../constants/address.js");
const account_js_1 = require("../../../errors/account.js");
const fromHex_js_1 = require("../../../utils/encoding/fromHex.js");
const toHex_js_1 = require("../../../utils/encoding/toHex.js");
async function getAssets(client, ...[parameters]) {
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
                    : (asset.address ?? address_js_1.ethAddress);
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
function formatRequest(parameters = {}) {
    const { account: account_, assets, assetTypes, chainIds } = parameters;
    if (typeof account_ === 'undefined')
        throw new account_js_1.AccountNotFoundError({
            docsPath: '/experimental/erc7811/getAssets',
        });
    const account = (0, parseAccount_js_1.parseAccount)(account_);
    return {
        account: account.address,
        assetFilter: assets,
        assetTypeFilter: assetTypes,
        chainFilter: chainIds?.map((chainId) => (0, toHex_js_1.numberToHex)(chainId)),
    };
}
function formatResponse(response) {
    return Object.fromEntries(Object.entries(response).map(([chainId, assets]) => [
        Number(chainId),
        assets.map((asset) => {
            const balance = (0, fromHex_js_1.hexToBigInt)(asset.balance);
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
                                ? { tokenId: (0, fromHex_js_1.hexToBigInt)(metadata.tokenId) }
                                : {}),
                        },
                    }
                    : {}),
            };
        }),
    ]));
}
//# sourceMappingURL=getAssets.js.map