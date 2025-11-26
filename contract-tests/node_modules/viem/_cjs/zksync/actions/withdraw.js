"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.withdraw = withdraw;
const account_js_1 = require("../../errors/account.js");
const index_js_1 = require("../../utils/index.js");
const abis_js_1 = require("../constants/abis.js");
const address_js_1 = require("../constants/address.js");
const getDefaultBridgeAddresses_js_1 = require("./getDefaultBridgeAddresses.js");
const getL2TokenAddress_js_1 = require("./getL2TokenAddress.js");
const sendTransaction_js_1 = require("./sendTransaction.js");
async function withdraw(client, parameters) {
    let { account: account_ = client.account, token = address_js_1.l2BaseTokenAddress, to, amount, bridgeAddress, ...rest } = parameters;
    const account = account_ ? (0, index_js_1.parseAccount)(account_) : client.account;
    if (!account)
        throw new account_js_1.AccountNotFoundError({
            docsPath: '/docs/actions/wallet/sendTransaction',
        });
    if (!to)
        to = account.address;
    let data;
    let contract;
    let value = 0n;
    if ((0, index_js_1.isAddressEqual)(token, address_js_1.legacyEthAddress) ||
        (0, index_js_1.isAddressEqual)(token, address_js_1.ethAddressInContracts))
        token = await (0, getL2TokenAddress_js_1.getL2TokenAddress)(client, { token: address_js_1.ethAddressInContracts });
    if ((0, index_js_1.isAddressEqual)(token, address_js_1.l2BaseTokenAddress)) {
        data = (0, index_js_1.encodeFunctionData)({
            abi: abis_js_1.ethTokenAbi,
            functionName: 'withdraw',
            args: [to],
        });
        value = amount;
        contract = address_js_1.l2BaseTokenAddress;
    }
    else {
        data = (0, index_js_1.encodeFunctionData)({
            abi: abis_js_1.l2SharedBridgeAbi,
            functionName: 'withdraw',
            args: [to, token, amount],
        });
        contract = bridgeAddress
            ? bridgeAddress
            : (await (0, getDefaultBridgeAddresses_js_1.getDefaultBridgeAddresses)(client)).sharedL2;
    }
    return await (0, sendTransaction_js_1.sendTransaction)(client, {
        account,
        to: contract,
        data,
        value,
        ...rest,
    });
}
//# sourceMappingURL=withdraw.js.map