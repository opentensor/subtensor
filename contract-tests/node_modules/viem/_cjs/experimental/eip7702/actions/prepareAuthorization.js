"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.prepareAuthorization = prepareAuthorization;
const parseAccount_js_1 = require("../../../accounts/utils/parseAccount.js");
const getChainId_js_1 = require("../../../actions/public/getChainId.js");
const getTransactionCount_js_1 = require("../../../actions/public/getTransactionCount.js");
const account_js_1 = require("../../../errors/account.js");
const isAddressEqual_js_1 = require("../../../utils/address/isAddressEqual.js");
const getAction_js_1 = require("../../../utils/getAction.js");
async function prepareAuthorization(client, parameters) {
    const { account: account_ = client.account, contractAddress, chainId, nonce, } = parameters;
    if (!account_)
        throw new account_js_1.AccountNotFoundError({
            docsPath: '/experimental/eip7702/prepareAuthorization',
        });
    const account = (0, parseAccount_js_1.parseAccount)(account_);
    const sponsor = (() => {
        const sponsor_ = parameters.sponsor ?? parameters.delegate;
        if (typeof sponsor_ === 'boolean')
            return sponsor_;
        if (sponsor_)
            return (0, parseAccount_js_1.parseAccount)(sponsor_);
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
                (await (0, getAction_js_1.getAction)(client, getChainId_js_1.getChainId, 'getChainId')({}));
    if (typeof authorization.nonce === 'undefined') {
        authorization.nonce = await (0, getAction_js_1.getAction)(client, getTransactionCount_js_1.getTransactionCount, 'getTransactionCount')({
            address: account.address,
            blockTag: 'pending',
        });
        if (!sponsor ||
            (sponsor !== true && (0, isAddressEqual_js_1.isAddressEqual)(account.address, sponsor.address)))
            authorization.nonce += 1;
    }
    return authorization;
}
//# sourceMappingURL=prepareAuthorization.js.map