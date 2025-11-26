"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.requestExecute = requestExecute;
const generatePrivateKey_js_1 = require("../../accounts/generatePrivateKey.js");
const privateKeyToAddress_js_1 = require("../../accounts/utils/privateKeyToAddress.js");
const readContract_js_1 = require("../../actions/public/readContract.js");
const sendTransaction_js_1 = require("../../actions/wallet/sendTransaction.js");
const public_js_1 = require("../../clients/decorators/public.js");
const account_js_1 = require("../../errors/account.js");
const chain_js_1 = require("../../errors/chain.js");
const index_js_1 = require("../../utils/index.js");
const abis_js_1 = require("../constants/abis.js");
const address_js_1 = require("../constants/address.js");
const number_js_1 = require("../constants/number.js");
const bridge_js_1 = require("../errors/bridge.js");
const estimateGasL1ToL2_js_1 = require("./estimateGasL1ToL2.js");
const getBridgehubContractAddress_js_1 = require("./getBridgehubContractAddress.js");
async function requestExecute(client, parameters) {
    let { account: account_ = client.account, chain: chain_ = client.chain, client: l2Client, contractAddress, calldata, l2Value = 0n, mintValue = 0n, operatorTip = 0n, factoryDeps = [], gasPerPubdataByte = number_js_1.requiredL1ToL2GasPerPubdataLimit, refundRecipient, l2GasLimit, value, gasPrice, maxFeePerGas, maxPriorityFeePerGas, ...rest } = parameters;
    const account = account_ ? (0, index_js_1.parseAccount)(account_) : client.account;
    if (!account)
        throw new account_js_1.AccountNotFoundError({
            docsPath: '/docs/actions/wallet/sendTransaction',
        });
    if (!l2Client.chain)
        throw new chain_js_1.ClientChainNotConfiguredError();
    const bridgehub = await (0, getBridgehubContractAddress_js_1.getBridgehubContractAddress)(l2Client);
    const baseToken = await (0, readContract_js_1.readContract)(client, {
        address: bridgehub,
        abi: abis_js_1.bridgehubAbi,
        functionName: 'baseToken',
        args: [BigInt(l2Client.chain.id)],
    });
    const isETHBasedChain = (0, index_js_1.isAddressEqual)(baseToken, address_js_1.ethAddressInContracts);
    refundRecipient ??= account.address;
    l2GasLimit ??= await (0, estimateGasL1ToL2_js_1.estimateGasL1ToL2)(l2Client, {
        chain: l2Client.chain,
        account: l2Client.account ??
            (0, index_js_1.parseAccount)((0, privateKeyToAddress_js_1.privateKeyToAddress)((0, generatePrivateKey_js_1.generatePrivateKey)())),
        data: calldata,
        to: contractAddress,
        value: l2Value,
        gasPerPubdata: gasPerPubdataByte,
        factoryDeps,
    });
    let gasPriceForEstimation = maxFeePerGas || gasPrice;
    if (!gasPriceForEstimation) {
        const estimatedFee = await getFeePrice(client);
        gasPriceForEstimation = estimatedFee.maxFeePerGas;
        maxFeePerGas = estimatedFee.maxFeePerGas;
        maxPriorityFeePerGas ??= estimatedFee.maxPriorityFeePerGas;
    }
    const baseCost = await (0, readContract_js_1.readContract)(client, {
        address: bridgehub,
        abi: abis_js_1.bridgehubAbi,
        functionName: 'l2TransactionBaseCost',
        args: [
            BigInt(l2Client.chain.id),
            gasPriceForEstimation,
            l2GasLimit,
            gasPerPubdataByte,
        ],
    });
    const l2Costs = baseCost + operatorTip + l2Value;
    let providedValue = isETHBasedChain ? value : mintValue;
    if (!providedValue || providedValue === 0n) {
        providedValue = l2Costs;
    }
    if (baseCost > providedValue)
        throw new bridge_js_1.BaseFeeHigherThanValueError(baseCost, providedValue);
    const data = (0, index_js_1.encodeFunctionData)({
        abi: abis_js_1.bridgehubAbi,
        functionName: 'requestL2TransactionDirect',
        args: [
            {
                chainId: BigInt(l2Client.chain.id),
                mintValue: providedValue,
                l2Contract: contractAddress,
                l2Value: l2Value,
                l2Calldata: calldata,
                l2GasLimit: l2GasLimit,
                l2GasPerPubdataByteLimit: gasPerPubdataByte,
                factoryDeps: factoryDeps,
                refundRecipient: refundRecipient,
            },
        ],
    });
    return await (0, sendTransaction_js_1.sendTransaction)(client, {
        chain: chain_,
        account: account,
        to: bridgehub,
        value: isETHBasedChain ? providedValue : value,
        data,
        gasPrice,
        maxFeePerGas,
        maxPriorityFeePerGas,
        ...rest,
    });
}
async function getFeePrice(client) {
    const client_ = client.extend(public_js_1.publicActions);
    const block = await client_.getBlock();
    const baseFee = typeof block.baseFeePerGas !== 'bigint'
        ? await client_.getGasPrice()
        : block.baseFeePerGas;
    const maxPriorityFeePerGas = await client_.estimateMaxPriorityFeePerGas();
    return {
        maxFeePerGas: (baseFee * 3n) / 2n + maxPriorityFeePerGas,
        maxPriorityFeePerGas: maxPriorityFeePerGas,
    };
}
//# sourceMappingURL=requestExecute.js.map