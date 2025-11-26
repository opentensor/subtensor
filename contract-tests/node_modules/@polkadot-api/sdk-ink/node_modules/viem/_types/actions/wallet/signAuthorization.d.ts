import type { Account } from '../../accounts/types.js';
import { type ParseAccountErrorType } from '../../accounts/utils/parseAccount.js';
import type { SignAuthorizationErrorType as SignAuthorizationErrorType_account, SignAuthorizationReturnType as SignAuthorizationReturnType_account } from '../../accounts/utils/signAuthorization.js';
import type { Client } from '../../clients/createClient.js';
import type { Transport } from '../../clients/transports/createTransport.js';
import { type AccountNotFoundErrorType, type AccountTypeNotSupportedErrorType } from '../../errors/account.js';
import type { ErrorType } from '../../errors/utils.js';
import type { Chain } from '../../types/chain.js';
import { type PrepareAuthorizationErrorType, type PrepareAuthorizationParameters } from './prepareAuthorization.js';
export type SignAuthorizationParameters<account extends Account | undefined = Account | undefined> = PrepareAuthorizationParameters<account>;
export type SignAuthorizationReturnType = SignAuthorizationReturnType_account;
export type SignAuthorizationErrorType = ParseAccountErrorType | AccountNotFoundErrorType | AccountTypeNotSupportedErrorType | PrepareAuthorizationErrorType | SignAuthorizationErrorType_account | ErrorType;
/**
 * Signs an [EIP-7702 Authorization](https://eips.ethereum.org/EIPS/eip-7702) object.
 *
 * With the calculated signature, you can:
 * - use [`verifyAuthorization`](https://viem.sh/docs/eip7702/verifyAuthorization) to verify the signed Authorization object,
 * - use [`recoverAuthorizationAddress`](https://viem.sh/docs/eip7702/recoverAuthorizationAddress) to recover the signing address from the signed Authorization object.
 *
 * @param client - Client to use
 * @param parameters - {@link SignAuthorizationParameters}
 * @returns The signed Authorization object. {@link SignAuthorizationReturnType}
 *
 * @example
 * import { createClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { mainnet } from 'viem/chains'
 * import { signAuthorization } from 'viem/experimental'
 *
 * const client = createClient({
 *   chain: mainnet,
 *   transport: http(),
 * })
 * const signature = await signAuthorization(client, {
 *   account: privateKeyToAccount('0x..'),
 *   contractAddress: '0xA0Cf798816D4b9b9866b5330EEa46a18382f251e',
 * })
 *
 * @example
 * // Account Hoisting
 * import { createClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { mainnet } from 'viem/chains'
 * import { signAuthorization } from 'viem/experimental'
 *
 * const client = createClient({
 *   account: privateKeyToAccount('0x…'),
 *   chain: mainnet,
 *   transport: http(),
 * })
 * const signature = await signAuthorization(client, {
 *   contractAddress: '0xA0Cf798816D4b9b9866b5330EEa46a18382f251e',
 * })
 */
export declare function signAuthorization<chain extends Chain | undefined, account extends Account | undefined>(client: Client<Transport, chain, account>, parameters: SignAuthorizationParameters<account>): Promise<SignAuthorizationReturnType>;
//# sourceMappingURL=signAuthorization.d.ts.map