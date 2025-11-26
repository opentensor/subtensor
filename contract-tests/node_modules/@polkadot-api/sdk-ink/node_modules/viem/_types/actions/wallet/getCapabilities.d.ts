import type { Address } from 'abitype';
import type { Client } from '../../clients/createClient.js';
import type { Transport } from '../../clients/transports/createTransport.js';
import type { ErrorType } from '../../errors/utils.js';
import type { Account } from '../../types/account.js';
import type { Capabilities, ChainIdToCapabilities, ExtractCapabilities } from '../../types/capabilities.js';
import type { Prettify } from '../../types/utils.js';
import type { RequestErrorType } from '../../utils/buildRequest.js';
export type GetCapabilitiesParameters<chainId extends number | undefined = undefined> = {
    account?: Account | Address | undefined;
    chainId?: chainId | number | undefined;
};
export type GetCapabilitiesReturnType<chainId extends number | undefined = undefined> = Prettify<chainId extends number ? ExtractCapabilities<'getCapabilities', 'ReturnType'> : ChainIdToCapabilities<Capabilities<ExtractCapabilities<'getCapabilities', 'ReturnType'>>, number>>;
export type GetCapabilitiesErrorType = RequestErrorType | ErrorType;
/**
 * Extract capabilities that a connected wallet supports (e.g. paymasters, session keys, etc).
 *
 * - Docs: https://viem.sh/docs/actions/wallet/getCapabilities
 * - JSON-RPC Methods: [`wallet_getCapabilities`](https://eips.ethereum.org/EIPS/eip-5792)
 *
 * @param client - Client to use
 * @returns The wallet's capabilities. {@link GetCapabilitiesReturnType}
 *
 * @example
 * import { createWalletClient, custom } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { getCapabilities } from 'viem/actions'
 *
 * const client = createWalletClient({
 *   chain: mainnet,
 *   transport: custom(window.ethereum),
 * })
 * const capabilities = await getCapabilities(client)
 */
export declare function getCapabilities<chainId extends number | undefined = undefined>(client: Client<Transport>, parameters?: GetCapabilitiesParameters<chainId>): Promise<GetCapabilitiesReturnType<chainId>>;
//# sourceMappingURL=getCapabilities.d.ts.map