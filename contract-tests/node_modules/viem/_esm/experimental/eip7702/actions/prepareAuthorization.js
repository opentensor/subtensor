import { parseAccount, } from '../../../accounts/utils/parseAccount.js';
import { getChainId } from '../../../actions/public/getChainId.js';
import { getTransactionCount } from '../../../actions/public/getTransactionCount.js';
import { AccountNotFoundError, } from '../../../errors/account.js';
import { isAddressEqual } from '../../../utils/address/isAddressEqual.js';
import { getAction } from '../../../utils/getAction.js';
/**
 * Prepares an [EIP-7702 Authorization](https://eips.ethereum.org/EIPS/eip-7702) object for signing.
 * This Action will fill the required fields of the Authorization object if they are not provided (e.g. `nonce` and `chainId`).
 *
 * With the prepared Authorization object, you can use [`signAuthorization`](https://viem.sh/experimental/eip7702/signAuthorization) to sign over the Authorization object.
 *
 * @param client - Client to use
 * @param parameters - {@link PrepareAuthorizationParameters}
 * @returns The prepared Authorization object. {@link PrepareAuthorizationReturnType}
 *
 * @example
 * import { createClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { mainnet } from 'viem/chains'
 * import { prepareAuthorization } from 'viem/experimental'
 *
 * const client = createClient({
 *   chain: mainnet,
 *   transport: http(),
 * })
 * const authorization = await prepareAuthorization(client, {
 *   account: privateKeyToAccount('0x..'),
 *   contractAddress: '0xA0Cf798816D4b9b9866b5330EEa46a18382f251e',
 * })
 *
 * @example
 * // Account Hoisting
 * import { createClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { mainnet } from 'viem/chains'
 * import { prepareAuthorization } from 'viem/experimental'
 *
 * const client = createClient({
 *   account: privateKeyToAccount('0x…'),
 *   chain: mainnet,
 *   transport: http(),
 * })
 * const authorization = await prepareAuthorization(client, {
 *   contractAddress: '0xA0Cf798816D4b9b9866b5330EEa46a18382f251e',
 * })
 */
export async function prepareAuthorization(client, parameters) {
    const { account: account_ = client.account, contractAddress, chainId, nonce, } = parameters;
    if (!account_)
        throw new AccountNotFoundError({
            docsPath: '/experimental/eip7702/prepareAuthorization',
        });
    const account = parseAccount(account_);
    const sponsor = (() => {
        const sponsor_ = parameters.sponsor ?? parameters.delegate;
        if (typeof sponsor_ === 'boolean')
            return sponsor_;
        if (sponsor_)
            return parseAccount(sponsor_);
        return undefined;
    })();
    const authorization = {
        contractAddress,
        chainId,
        nonce,
    };
    if (typeof authorization.chainId === 'undefined')
        authorization.chainId =
            client.chain?.id ??
                (await getAction(client, getChainId, 'getChainId')({}));
    if (typeof authorization.nonce === 'undefined') {
        authorization.nonce = await getAction(client, getTransactionCount, 'getTransactionCount')({
            address: account.address,
            blockTag: 'pending',
        });
        if (!sponsor ||
            (sponsor !== true && isAddressEqual(account.address, sponsor.address)))
            authorization.nonce += 1;
    }
    return authorization;
}
//# sourceMappingURL=prepareAuthorization.js.map