import { parseAccount, } from '../../accounts/utils/parseAccount.js';
import { AccountNotFoundError, AccountTypeNotSupportedError, } from '../../errors/account.js';
import { prepareAuthorization, } from './prepareAuthorization.js';
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
export async function signAuthorization(client, parameters) {
    const { account: account_ = client.account } = parameters;
    if (!account_)
        throw new AccountNotFoundError({
            docsPath: '/docs/eip7702/signAuthorization',
        });
    const account = parseAccount(account_);
    if (!account.signAuthorization)
        throw new AccountTypeNotSupportedError({
            docsPath: '/docs/eip7702/signAuthorization',
            metaMessages: [
                'The `signAuthorization` Action does not support JSON-RPC Accounts.',
            ],
            type: account.type,
        });
    const authorization = await prepareAuthorization(client, parameters);
    return account.signAuthorization(authorization);
}
//# sourceMappingURL=signAuthorization.js.map