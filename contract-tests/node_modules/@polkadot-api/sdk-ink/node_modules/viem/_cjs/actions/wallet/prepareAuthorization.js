"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.prepareAuthorization = prepareAuthorization;
const parseAccount_js_1 = require("../../accounts/utils/parseAccount.js");
const account_js_1 = require("../../errors/account.js");
const isAddressEqual_js_1 = require("../../utils/address/isAddressEqual.js");
const getAction_js_1 = require("../../utils/getAction.js");
const getChainId_js_1 = require("../public/getChainId.js");
const getTransactionCount_js_1 = require("../public/getTransactionCount.js");
async function prepareAuthorization(client, parameters) {
    const { account: account_ = client.account, chainId, nonce } = parameters;
    if (!account_)
        throw new account_js_1.AccountNotFoundError({
            docsPath: '/docs/eip7702/prepareAuthorization',
        });
    const account = (0, parseAccount_js_1.parseAccount)(account_);
    const executor = (() => {
        if (!parameters.executor)
            return undefined;
        if (parameters.executor === 'self')
            return parameters.executor;
        return (0, parseAccount_js_1.parseAccount)(parameters.executor);
    })();
    const authorization = {
        address: parameters.contractAddress ?? parameters.address,
        chainId,
        nonce,
    };
    if (typeof authorization.chainId === 'undefined')
        authorization.chainId =
            client.chain?.id ??
                (await (0, getAction_js_1.getAction)(client, getChainId_js_1.getChainId, 'getChainId')({}));
    if (typeof authorization.nonce === 'undefined') {
        authorization.nonce = await (0, getAction_js_1.getAction)(client, getTransactionCount_js_1.getTransactionCount, 'getTransactionCount')({
            address: account.address,
            blockTag: 'pending',
        });
        if (executor === 'self' ||
            (executor?.address && (0, isAddressEqual_js_1.isAddressEqual)(executor.address, account.address)))
            authorization.nonce += 1;
    }
    return authorization;
}
//# sourceMappingURL=prepareAuthorization.js.map