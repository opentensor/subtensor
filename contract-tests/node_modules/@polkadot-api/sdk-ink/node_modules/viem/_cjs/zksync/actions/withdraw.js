"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.withdraw = withdraw;
const abitype_1 = require("abitype");
const readContract_js_1 = require("../../actions/public/readContract.js");
const account_js_1 = require("../../errors/account.js");
const chain_js_1 = require("../../errors/chain.js");
const index_js_1 = require("../../utils/index.js");
const abis_js_1 = require("../constants/abis.js");
const address_js_1 = require("../constants/address.js");
const getDefaultBridgeAddresses_js_1 = require("./getDefaultBridgeAddresses.js");
const getL1ChainId_js_1 = require("./getL1ChainId.js");
const getL2TokenAddress_js_1 = require("./getL2TokenAddress.js");
const sendTransaction_js_1 = require("./sendTransaction.js");
async function withdraw(client, parameters) {
    let { account: account_ = client.account, chain: chain_ = client.chain, token = address_js_1.l2BaseTokenAddress, to, amount, bridgeAddress, ...rest } = parameters;
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
        const assetId = await (0, readContract_js_1.readContract)(client, {
            address: address_js_1.l2NativeTokenVaultAddress,
            abi: (0, abitype_1.parseAbi)(['function assetId(address token) view returns (bytes32)']),
            functionName: 'assetId',
            args: [token],
        });
        const originChainId = await (0, readContract_js_1.readContract)(client, {
            address: address_js_1.l2NativeTokenVaultAddress,
            abi: (0, abitype_1.parseAbi)([
                'function originChainId(bytes32 assetId) view returns (uint256)',
            ]),
            functionName: 'originChainId',
            args: [assetId],
        });
        const l1ChainId = await (0, getL1ChainId_js_1.getL1ChainId)(client);
        const isTokenL1Native = originChainId === BigInt(l1ChainId) || token === address_js_1.ethAddressInContracts;
        if (!bridgeAddress) {
            bridgeAddress = isTokenL1Native
                ? (await (0, getDefaultBridgeAddresses_js_1.getDefaultBridgeAddresses)(client)).sharedL2
                : address_js_1.l2AssetRouterAddress;
        }
        if (!isTokenL1Native) {
            contract = address_js_1.l2AssetRouterAddress;
            if (!chain_)
                throw new chain_js_1.ClientChainNotConfiguredError();
            const chainId = chain_.id;
            const assetId = (0, index_js_1.keccak256)((0, index_js_1.encodeAbiParameters)([{ type: 'uint256' }, { type: 'address' }, { type: 'address' }], [BigInt(chainId), address_js_1.l2NativeTokenVaultAddress, token]));
            const assetData = (0, index_js_1.encodeAbiParameters)([{ type: 'uint256' }, { type: 'address' }, { type: 'address' }], [BigInt(amount), to, token]);
            data = (0, index_js_1.encodeFunctionData)({
                abi: (0, abitype_1.parseAbi)([
                    'function withdraw(bytes32 _assetId, bytes _transferData)',
                ]),
                functionName: 'withdraw',
                args: [assetId, assetData],
            });
        }
        else {
            contract = bridgeAddress;
            data = (0, index_js_1.encodeFunctionData)({
                abi: abis_js_1.l2SharedBridgeAbi,
                functionName: 'withdraw',
                args: [to, token, amount],
            });
        }
    }
    return await (0, sendTransaction_js_1.sendTransaction)(client, {
        chain: chain_,
        account,
        to: contract,
        data,
        value,
        ...rest,
    });
}
//# sourceMappingURL=withdraw.js.map