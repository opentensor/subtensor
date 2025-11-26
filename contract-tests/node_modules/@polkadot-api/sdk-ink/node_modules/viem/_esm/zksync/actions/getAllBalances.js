import { parseAccount } from '../../accounts/utils/parseAccount.js';
import { hexToBigInt } from '../../utils/encoding/fromHex.js';
/* @deprecated This method has been removed from the node API. */
export async function getAllBalances(client, parameters) {
    const { account: account_ } = parameters;
    const account = account_ ? parseAccount(account_) : client.account;
    const balances = await client.request({
        method: 'zks_getAllAccountBalances',
        params: [account.address],
    });
    const convertedBalances = {};
    for (const token in balances)
        convertedBalances[token] = hexToBigInt(balances[token]);
    return convertedBalances;
}
//# sourceMappingURL=getAllBalances.js.map