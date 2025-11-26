"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.signAuthorization = signAuthorization;
const parseAccount_js_1 = require("../../accounts/utils/parseAccount.js");
const account_js_1 = require("../../errors/account.js");
const prepareAuthorization_js_1 = require("./prepareAuthorization.js");
async function signAuthorization(client, parameters) {
    const { account: account_ = client.account } = parameters;
    if (!account_)
        throw new account_js_1.AccountNotFoundError({
            docsPath: '/docs/eip7702/signAuthorization',
        });
    const account = (0, parseAccount_js_1.parseAccount)(account_);
    if (!account.signAuthorization)
        throw new account_js_1.AccountTypeNotSupportedError({
            docsPath: '/docs/eip7702/signAuthorization',
            metaMessages: [
                'The `signAuthorization` Action does not support JSON-RPC Accounts.',
            ],
            type: account.type,
        });
    const authorization = await (0, prepareAuthorization_js_1.prepareAuthorization)(client, parameters);
    return account.signAuthorization(authorization);
}
//# sourceMappingURL=signAuthorization.js.map