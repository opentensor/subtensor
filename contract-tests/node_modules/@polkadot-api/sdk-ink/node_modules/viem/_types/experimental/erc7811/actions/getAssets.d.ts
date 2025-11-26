import type { Address } from 'abitype';
import { type ParseAccountErrorType } from '../../../accounts/utils/parseAccount.js';
import type { Client } from '../../../clients/createClient.js';
import type { Transport } from '../../../clients/transports/createTransport.js';
import type { ErrorType } from '../../../errors/utils.js';
import type { Account, GetAccountParameter } from '../../../types/account.js';
import type { Chain } from '../../../types/chain.js';
import type { OneOf, Prettify } from '../../../types/utils.js';
import { type HexToBigIntErrorType } from '../../../utils/encoding/fromHex.js';
import { type NumberToHexErrorType } from '../../../utils/encoding/toHex.js';
export type GetAssetsParameters<aggregate extends boolean | ((asset: getAssets.Asset) => string) | undefined = undefined, account extends Account | undefined = Account | undefined> = GetAccountParameter<account> & {
    /**
     * Whether or not to aggregate assets across multiple chains,
     * and assign them to a '0' key.
     * @default true
     */
    aggregate?: aggregate | boolean | ((asset: getAssets.Asset) => string) | undefined;
    /** Filter by assets. */
    assets?: {
        [chainId: number]: readonly ({
            address: 'native';
            type: 'native';
        } | {
            address: Address;
            type: getAssets.AssetType;
        })[];
    } | undefined;
    /** Filter by asset types. */
    assetTypes?: readonly getAssets.AssetType[] | undefined;
    /** Filter by chain IDs. */
    chainIds?: readonly number[] | undefined;
};
export type GetAssetsReturnType<aggregate extends boolean | ((asset: getAssets.Asset) => string) | undefined = undefined> = {
    [chainId: number]: readonly getAssets.Asset<false>[];
} & (aggregate extends false ? {} : {
    0: readonly getAssets.Asset<true>[];
});
export type GetAssetsErrorType = HexToBigIntErrorType | NumberToHexErrorType | ParseAccountErrorType | ErrorType;
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
export declare function getAssets<chain extends Chain | undefined, account extends Account | undefined = Account | undefined, aggregate extends boolean | ((asset: getAssets.Asset) => string) | undefined = undefined>(client: Client<Transport, chain, account>, ...[parameters]: account extends Account ? [GetAssetsParameters<aggregate, account>] | [] : [GetAssetsParameters<aggregate, account>]): Promise<Prettify<GetAssetsReturnType<aggregate>>>;
export declare namespace getAssets {
    type Asset<chainIds extends boolean = false> = OneOf<CustomAsset | Erc20Asset | Erc721Asset | NativeAsset> & (chainIds extends true ? {
        chainIds: readonly number[];
    } : {});
    type AssetType = 'native' | 'erc20' | 'erc721' | (string & {});
    type CustomAsset = {
        address: Address;
        balance: bigint;
        metadata: {
            [key: string]: unknown;
        };
        type: {
            custom: string;
        };
    };
    type Erc20Asset = {
        address: Address;
        balance: bigint;
        metadata: {
            name: string;
            symbol: string;
            decimals: number;
        };
        type: 'erc20';
    };
    type Erc721Asset = {
        address: Address;
        balance: bigint;
        metadata: {
            name: string;
            symbol: string;
            tokenId: bigint;
            tokenUri?: string | undefined;
        };
        type: 'erc721';
    };
    type NativeAsset = {
        balance: bigint;
        type: 'native';
    };
}
//# sourceMappingURL=getAssets.d.ts.map