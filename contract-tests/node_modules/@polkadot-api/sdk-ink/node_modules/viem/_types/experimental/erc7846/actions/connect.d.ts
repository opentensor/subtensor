import type { Address } from 'abitype';
import { type RequestAddressesErrorType } from '../../../actions/wallet/requestAddresses.js';
import type { Client } from '../../../clients/createClient.js';
import type { Transport } from '../../../clients/transports/createTransport.js';
import type { ExtractCapabilities } from '../../../types/capabilities.js';
import type { Chain } from '../../../types/chain.js';
import type { Prettify } from '../../../types/utils.js';
import type { RequestErrorType } from '../../../utils/buildRequest.js';
export type ConnectParameters = Prettify<{
    capabilities?: ExtractCapabilities<'connect', 'Request'> | undefined;
}>;
export type ConnectReturnType = Prettify<{
    accounts: readonly {
        address: Address;
        capabilities?: ExtractCapabilities<'connect', 'ReturnType'> | undefined;
    }[];
}>;
export type ConnectErrorType = RequestErrorType | RequestAddressesErrorType;
/**
 * Requests to connect account(s) with optional capabilities.
 *
 * - Docs: https://viem.sh/experimental/erc7846/connect
 * - JSON-RPC Methods: [`wallet_connect`](https://github.com/ethereum/ERCs/blob/abd1c9f4eda2d6ad06ade0e3af314637a27d1ee7/ERCS/erc-7846.md)
 *
 * @param client - Client to use
 * @param parameters - {@link ConnectParameters}
 * @returns List of accounts managed by a wallet {@link ConnectReturnType}
 *
 * @example
 * import { createWalletClient, custom } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { connect } from 'viem/experimental/erc7846'
 *
 * const client = createWalletClient({
 *   chain: mainnet,
 *   transport: custom(window.ethereum),
 * })
 * const response = await connect(client)
 */
export declare function connect<chain extends Chain | undefined>(client: Client<Transport, chain>, parameters?: ConnectParameters): Promise<ConnectReturnType>;
//# sourceMappingURL=connect.d.ts.map