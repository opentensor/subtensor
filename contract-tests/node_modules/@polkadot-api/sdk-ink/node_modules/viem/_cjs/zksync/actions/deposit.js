"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.deposit = deposit;
const abitype_1 = require("abitype");
const estimateGas_js_1 = require("../../actions/public/estimateGas.js");
const readContract_js_1 = require("../../actions/public/readContract.js");
const waitForTransactionReceipt_js_1 = require("../../actions/public/waitForTransactionReceipt.js");
const sendTransaction_js_1 = require("../../actions/wallet/sendTransaction.js");
const writeContract_js_1 = require("../../actions/wallet/writeContract.js");
const public_js_1 = require("../../clients/decorators/public.js");
const abis_js_1 = require("../../constants/abis.js");
const address_js_1 = require("../../constants/address.js");
const bytes_js_1 = require("../../constants/bytes.js");
const account_js_1 = require("../../errors/account.js");
const chain_js_1 = require("../../errors/chain.js");
const index_js_1 = require("../../utils/index.js");
const abis_js_2 = require("../constants/abis.js");
const address_js_2 = require("../constants/address.js");
const number_js_1 = require("../constants/number.js");
const bridge_js_1 = require("../errors/bridge.js");
const applyL1ToL2Alias_js_1 = require("../utils/bridge/applyL1ToL2Alias.js");
const estimateGasL1ToL2_js_1 = require("./estimateGasL1ToL2.js");
const getBridgehubContractAddress_js_1 = require("./getBridgehubContractAddress.js");
const getDefaultBridgeAddresses_js_1 = require("./getDefaultBridgeAddresses.js");
const getL1Allowance_js_1 = require("./getL1Allowance.js");
async function deposit(client, parameters) {
    let { account: account_ = client.account, chain: chain_ = client.chain, client: l2Client, token, amount, approveToken, approveBaseToken, gas, } = parameters;
    const account = account_ ? (0, index_js_1.parseAccount)(account_) : client.account;
    if (!account)
        throw new account_js_1.AccountNotFoundError({
            docsPath: '/docs/actions/wallet/sendTransaction',
        });
    if (!l2Client.chain)
        throw new chain_js_1.ClientChainNotConfiguredError();
    if ((0, index_js_1.isAddressEqual)(token, address_js_2.legacyEthAddress))
        token = address_js_2.ethAddressInContracts;
    const bridgeAddresses = await getBridgeAddresses(client, l2Client);
    const bridgehub = await (0, getBridgehubContractAddress_js_1.getBridgehubContractAddress)(l2Client);
    const baseToken = await (0, readContract_js_1.readContract)(client, {
        address: bridgehub,
        abi: abis_js_2.bridgehubAbi,
        functionName: 'baseToken',
        args: [BigInt(l2Client.chain.id)],
    });
    const { mintValue, tx } = await getL1DepositTx(client, account, { ...parameters, token }, bridgeAddresses, bridgehub, baseToken);
    await approveTokens(client, chain_, tx.bridgeAddress, baseToken, mintValue, account, token, amount, approveToken, approveBaseToken);
    if (!gas) {
        const baseGasLimit = await (0, estimateGas_js_1.estimateGas)(client, {
            account: account.address,
            to: bridgehub,
            value: tx.value,
            data: tx.data,
        });
        gas = scaleGasLimit(baseGasLimit);
    }
    return await (0, sendTransaction_js_1.sendTransaction)(client, {
        chain: chain_,
        account,
        gas,
        ...tx,
    });
}
async function getL1DepositTx(client, account, parameters, bridgeAddresses, bridgehub, baseToken) {
    let { account: _account, chain: _chain, client: l2Client, token, amount, to, operatorTip = 0n, l2GasLimit, gasPerPubdataByte = number_js_1.requiredL1ToL2GasPerPubdataLimit, refundRecipient = address_js_1.zeroAddress, bridgeAddress, customBridgeData, value, gasPrice, maxFeePerGas, maxPriorityFeePerGas, approveToken: _approveToken, approveBaseToken: _approveBaseToken, ...rest } = parameters;
    if (!l2Client.chain)
        throw new chain_js_1.ClientChainNotConfiguredError();
    to ??= account.address;
    let gasPriceForEstimation = maxFeePerGas || gasPrice;
    if (!gasPriceForEstimation) {
        const estimatedFee = await getFeePrice(client);
        gasPriceForEstimation = estimatedFee.maxFeePerGas;
        maxFeePerGas = estimatedFee.maxFeePerGas;
        maxPriorityFeePerGas ??= estimatedFee.maxPriorityFeePerGas;
    }
    const { l2GasLimit_, baseCost } = await getL2BridgeTxFeeParams(client, l2Client, bridgehub, gasPriceForEstimation, account.address, token, amount, to, gasPerPubdataByte, baseToken, l2GasLimit, bridgeAddress, customBridgeData);
    l2GasLimit = l2GasLimit_;
    let mintValue;
    let data;
    const isETHBasedChain = (0, index_js_1.isAddressEqual)(baseToken, address_js_2.ethAddressInContracts);
    if ((isETHBasedChain && (0, index_js_1.isAddressEqual)(token, address_js_2.ethAddressInContracts)) ||
        (0, index_js_1.isAddressEqual)(token, baseToken)) {
        mintValue = baseCost + operatorTip + amount;
        let providedValue = isETHBasedChain ? value : mintValue;
        if (!providedValue || providedValue === 0n)
            providedValue = mintValue;
        if (baseCost > providedValue)
            throw new bridge_js_1.BaseFeeHigherThanValueError(baseCost, providedValue);
        value = isETHBasedChain ? providedValue : 0n;
        bridgeAddress = bridgeAddresses.sharedL1;
        data = (0, index_js_1.encodeFunctionData)({
            abi: abis_js_2.bridgehubAbi,
            functionName: 'requestL2TransactionDirect',
            args: [
                {
                    chainId: BigInt(l2Client.chain.id),
                    mintValue: providedValue,
                    l2Contract: to,
                    l2Value: amount,
                    l2Calldata: '0x',
                    l2GasLimit,
                    l2GasPerPubdataByteLimit: gasPerPubdataByte,
                    factoryDeps: [],
                    refundRecipient,
                },
            ],
        });
    }
    else if ((0, index_js_1.isAddressEqual)(baseToken, address_js_2.ethAddressInContracts)) {
        mintValue = baseCost + BigInt(operatorTip);
        value = mintValue;
        if (baseCost > mintValue)
            throw new bridge_js_1.BaseFeeHigherThanValueError(baseCost, mintValue);
        bridgeAddress ??= bridgeAddresses.sharedL1;
        data = (0, index_js_1.encodeFunctionData)({
            abi: abis_js_2.bridgehubAbi,
            functionName: 'requestL2TransactionTwoBridges',
            args: [
                {
                    chainId: BigInt(l2Client.chain.id),
                    mintValue,
                    l2Value: 0n,
                    l2GasLimit,
                    l2GasPerPubdataByteLimit: gasPerPubdataByte,
                    refundRecipient,
                    secondBridgeAddress: bridgeAddress,
                    secondBridgeValue: 0n,
                    secondBridgeCalldata: await getSecondBridgeCalldata(client, bridgeAddresses.l1NativeTokenVault, token, amount, to),
                },
            ],
        });
    }
    else if ((0, index_js_1.isAddressEqual)(token, address_js_2.ethAddressInContracts)) {
        mintValue = baseCost + operatorTip;
        value = amount;
        if (baseCost > mintValue)
            throw new bridge_js_1.BaseFeeHigherThanValueError(baseCost, mintValue);
        bridgeAddress = bridgeAddresses.sharedL1;
        data = (0, index_js_1.encodeFunctionData)({
            abi: abis_js_2.bridgehubAbi,
            functionName: 'requestL2TransactionTwoBridges',
            args: [
                {
                    chainId: BigInt(l2Client.chain.id),
                    mintValue,
                    l2Value: 0n,
                    l2GasLimit,
                    l2GasPerPubdataByteLimit: gasPerPubdataByte,
                    refundRecipient,
                    secondBridgeAddress: bridgeAddress,
                    secondBridgeValue: amount,
                    secondBridgeCalldata: await getSecondBridgeCalldata(client, bridgeAddresses.l1NativeTokenVault, address_js_2.ethAddressInContracts, amount, to),
                },
            ],
        });
    }
    else {
        mintValue = baseCost + operatorTip;
        value ??= 0n;
        if (baseCost > mintValue)
            throw new bridge_js_1.BaseFeeHigherThanValueError(baseCost, mintValue);
        bridgeAddress ??= bridgeAddresses.sharedL1;
        data = (0, index_js_1.encodeFunctionData)({
            abi: abis_js_2.bridgehubAbi,
            functionName: 'requestL2TransactionTwoBridges',
            args: [
                {
                    chainId: BigInt(l2Client.chain.id),
                    mintValue,
                    l2Value: 0n,
                    l2GasLimit,
                    l2GasPerPubdataByteLimit: gasPerPubdataByte,
                    refundRecipient,
                    secondBridgeAddress: bridgeAddress,
                    secondBridgeValue: 0n,
                    secondBridgeCalldata: await getSecondBridgeCalldata(client, bridgeAddresses.l1NativeTokenVault, token, amount, to),
                },
            ],
        });
    }
    return {
        mintValue,
        tx: {
            bridgeAddress,
            to: bridgehub,
            data,
            value,
            gasPrice,
            maxFeePerGas,
            maxPriorityFeePerGas,
            ...rest,
        },
    };
}
async function approveTokens(client, chain, bridgeAddress, baseToken, mintValue, account, token, amount, approveToken, approveBaseToken) {
    if ((0, index_js_1.isAddressEqual)(baseToken, address_js_2.ethAddressInContracts)) {
        if (approveToken) {
            const overrides = typeof approveToken === 'boolean' ? {} : approveToken;
            const allowance = await (0, getL1Allowance_js_1.getL1Allowance)(client, {
                token,
                bridgeAddress,
                account,
            });
            if (allowance < amount) {
                const hash = await (0, writeContract_js_1.writeContract)(client, {
                    chain,
                    account,
                    address: token,
                    abi: abis_js_1.erc20Abi,
                    functionName: 'approve',
                    args: [bridgeAddress, amount],
                    ...overrides,
                });
                await (0, waitForTransactionReceipt_js_1.waitForTransactionReceipt)(client, { hash });
            }
        }
        return;
    }
    if ((0, index_js_1.isAddressEqual)(token, address_js_2.ethAddressInContracts)) {
        if (approveBaseToken) {
            const overrides = typeof approveToken === 'boolean' ? {} : approveToken;
            const allowance = await (0, getL1Allowance_js_1.getL1Allowance)(client, {
                token: baseToken,
                bridgeAddress,
                account,
            });
            if (allowance < mintValue) {
                const hash = await (0, writeContract_js_1.writeContract)(client, {
                    chain,
                    account,
                    address: baseToken,
                    abi: abis_js_1.erc20Abi,
                    functionName: 'approve',
                    args: [bridgeAddress, mintValue],
                    ...overrides,
                });
                await (0, waitForTransactionReceipt_js_1.waitForTransactionReceipt)(client, { hash });
            }
            return;
        }
    }
    if ((0, index_js_1.isAddressEqual)(token, baseToken)) {
        if (approveToken || approveBaseToken) {
            const overrides = typeof approveToken === 'boolean'
                ? {}
                : (approveToken ?? typeof approveBaseToken === 'boolean')
                    ? {}
                    : approveBaseToken;
            const allowance = await (0, getL1Allowance_js_1.getL1Allowance)(client, {
                token: baseToken,
                bridgeAddress,
                account,
            });
            if (allowance < mintValue) {
                const hash = await (0, writeContract_js_1.writeContract)(client, {
                    chain,
                    account,
                    address: baseToken,
                    abi: abis_js_1.erc20Abi,
                    functionName: 'approve',
                    args: [bridgeAddress, mintValue],
                    ...overrides,
                });
                await (0, waitForTransactionReceipt_js_1.waitForTransactionReceipt)(client, { hash });
            }
        }
        return;
    }
    if (approveBaseToken) {
        const overrides = typeof approveToken === 'boolean' ? {} : approveToken;
        const allowance = await (0, getL1Allowance_js_1.getL1Allowance)(client, {
            token: baseToken,
            bridgeAddress,
            account,
        });
        if (allowance < mintValue) {
            const hash = await (0, writeContract_js_1.writeContract)(client, {
                chain,
                account,
                address: baseToken,
                abi: abis_js_1.erc20Abi,
                functionName: 'approve',
                args: [bridgeAddress, mintValue],
                ...overrides,
            });
            await (0, waitForTransactionReceipt_js_1.waitForTransactionReceipt)(client, { hash });
        }
    }
    if (approveToken) {
        const overrides = typeof approveToken === 'boolean' ? {} : approveToken;
        const allowance = await (0, getL1Allowance_js_1.getL1Allowance)(client, {
            token,
            bridgeAddress,
            account,
        });
        if (allowance < amount) {
            const hash = await (0, writeContract_js_1.writeContract)(client, {
                chain,
                account,
                address: token,
                abi: abis_js_1.erc20Abi,
                functionName: 'approve',
                args: [bridgeAddress, amount],
                ...overrides,
            });
            await (0, waitForTransactionReceipt_js_1.waitForTransactionReceipt)(client, { hash });
        }
    }
}
async function getL2BridgeTxFeeParams(client, l2Client, bridgehub, gasPrice, from, token, amount, to, gasPerPubdataByte, baseToken, l2GasLimit, bridgeAddress, customBridgeData) {
    if (!l2Client.chain)
        throw new chain_js_1.ClientChainNotConfiguredError();
    let l2GasLimit_ = l2GasLimit;
    if (!l2GasLimit_)
        l2GasLimit_ = bridgeAddress
            ? await getL2GasLimitFromCustomBridge(client, l2Client, from, token, amount, to, gasPerPubdataByte, bridgeAddress, customBridgeData)
            : await getL2GasLimitFromDefaultBridge(client, l2Client, from, token, amount, to, gasPerPubdataByte, baseToken);
    const baseCost = await (0, readContract_js_1.readContract)(client, {
        address: bridgehub,
        abi: abis_js_2.bridgehubAbi,
        functionName: 'l2TransactionBaseCost',
        args: [BigInt(l2Client.chain.id), gasPrice, l2GasLimit_, gasPerPubdataByte],
    });
    return { l2GasLimit_, baseCost };
}
async function getL2GasLimitFromDefaultBridge(client, l2Client, from, token, amount, to, gasPerPubdataByte, baseToken) {
    if ((0, index_js_1.isAddressEqual)(token, baseToken)) {
        return await (0, estimateGasL1ToL2_js_1.estimateGasL1ToL2)(l2Client, {
            chain: l2Client.chain,
            account: from,
            from,
            to,
            value: amount,
            data: '0x',
            gasPerPubdata: gasPerPubdataByte,
        });
    }
    const value = 0n;
    const bridgeAddresses = await (0, getDefaultBridgeAddresses_js_1.getDefaultBridgeAddresses)(l2Client);
    const l1BridgeAddress = bridgeAddresses.sharedL1;
    const l2BridgeAddress = bridgeAddresses.sharedL2;
    const bridgeData = await encodeDefaultBridgeData(client, token);
    const calldata = (0, index_js_1.encodeFunctionData)({
        abi: (0, abitype_1.parseAbi)([
            'function finalizeDeposit(address _l1Sender, address _l2Receiver, address _l1Token, uint256 _amount, bytes _data)',
        ]),
        functionName: 'finalizeDeposit',
        args: [
            from,
            to,
            (0, index_js_1.isAddressEqual)(token, address_js_2.legacyEthAddress) ? address_js_2.ethAddressInContracts : token,
            amount,
            bridgeData,
        ],
    });
    return await (0, estimateGasL1ToL2_js_1.estimateGasL1ToL2)(l2Client, {
        chain: l2Client.chain,
        account: (0, applyL1ToL2Alias_js_1.applyL1ToL2Alias)(l1BridgeAddress),
        to: l2BridgeAddress,
        data: calldata,
        gasPerPubdata: gasPerPubdataByte,
        value,
    });
}
async function getL2GasLimitFromCustomBridge(client, l2Client, from, token, amount, to, gasPerPubdataByte, bridgeAddress, customBridgeData) {
    let customBridgeData_ = customBridgeData;
    if (!customBridgeData_ || customBridgeData_ === '0x')
        customBridgeData_ = await encodeDefaultBridgeData(client, token);
    const l2BridgeAddress = await (0, readContract_js_1.readContract)(client, {
        address: token,
        abi: (0, abitype_1.parseAbi)([
            'function l2BridgeAddress(uint256 _chainId) view returns (address)',
        ]),
        functionName: 'l2BridgeAddress',
        args: [BigInt(l2Client.chain.id)],
    });
    const calldata = (0, index_js_1.encodeFunctionData)({
        abi: (0, abitype_1.parseAbi)([
            'function finalizeDeposit(address _l1Sender, address _l2Receiver, address _l1Token, uint256 _amount, bytes _data)',
        ]),
        functionName: 'finalizeDeposit',
        args: [from, to, token, amount, customBridgeData_],
    });
    return await (0, estimateGasL1ToL2_js_1.estimateGasL1ToL2)(l2Client, {
        chain: l2Client.chain,
        account: from,
        from: (0, applyL1ToL2Alias_js_1.applyL1ToL2Alias)(bridgeAddress),
        to: l2BridgeAddress,
        data: calldata,
        gasPerPubdata: gasPerPubdataByte,
    });
}
async function encodeDefaultBridgeData(client, token) {
    let token_ = token;
    if ((0, index_js_1.isAddressEqual)(token, address_js_2.legacyEthAddress))
        token_ = address_js_2.ethAddressInContracts;
    let name = 'Ether';
    let symbol = 'ETH';
    let decimals = 18n;
    if (!(0, index_js_1.isAddressEqual)(token_, address_js_2.ethAddressInContracts)) {
        name = await (0, readContract_js_1.readContract)(client, {
            address: token_,
            abi: abis_js_1.erc20Abi,
            functionName: 'name',
            args: [],
        });
        symbol = await (0, readContract_js_1.readContract)(client, {
            address: token_,
            abi: abis_js_1.erc20Abi,
            functionName: 'symbol',
            args: [],
        });
        decimals = BigInt(await (0, readContract_js_1.readContract)(client, {
            address: token_,
            abi: abis_js_1.erc20Abi,
            functionName: 'decimals',
            args: [],
        }));
    }
    const nameBytes = (0, index_js_1.encodeAbiParameters)([{ type: 'string' }], [name]);
    const symbolBytes = (0, index_js_1.encodeAbiParameters)([{ type: 'string' }], [symbol]);
    const decimalsBytes = (0, index_js_1.encodeAbiParameters)([{ type: 'uint256' }], [decimals]);
    return (0, index_js_1.encodeAbiParameters)([{ type: 'bytes' }, { type: 'bytes' }, { type: 'bytes' }], [nameBytes, symbolBytes, decimalsBytes]);
}
async function getSecondBridgeCalldata(client, l1NativeTokenVault, token, amount, to) {
    let assetId;
    let token_ = token;
    if ((0, index_js_1.isAddressEqual)(token, address_js_2.legacyEthAddress))
        token_ = address_js_2.ethAddressInContracts;
    const assetIdFromNTV = await (0, readContract_js_1.readContract)(client, {
        address: l1NativeTokenVault,
        abi: (0, abitype_1.parseAbi)(['function assetId(address token) view returns (bytes32)']),
        functionName: 'assetId',
        args: [token_],
    });
    if (assetIdFromNTV && assetIdFromNTV !== bytes_js_1.zeroHash)
        assetId = assetIdFromNTV;
    else {
        if (!client.chain)
            throw new chain_js_1.ClientChainNotConfiguredError();
        assetId = (0, index_js_1.keccak256)((0, index_js_1.encodeAbiParameters)([{ type: 'uint256' }, { type: 'address' }, { type: 'address' }], [BigInt(client.chain.id), address_js_2.l2NativeTokenVaultAddress, token_]));
    }
    const ntvData = (0, index_js_1.encodeAbiParameters)([{ type: 'uint256' }, { type: 'address' }, { type: 'address' }], [BigInt(amount), to, token_]);
    const data = (0, index_js_1.encodeAbiParameters)([{ type: 'bytes32' }, { type: 'bytes' }], [assetId, ntvData]);
    return (0, index_js_1.concatHex)(['0x01', data]);
}
async function getBridgeAddresses(client, l2Client) {
    const addresses = await (0, getDefaultBridgeAddresses_js_1.getDefaultBridgeAddresses)(l2Client);
    let l1Nullifier = addresses.l1Nullifier;
    let l1NativeTokenVault = addresses.l1NativeTokenVault;
    if (!l1Nullifier)
        l1Nullifier = await (0, readContract_js_1.readContract)(client, {
            address: addresses.sharedL1,
            abi: (0, abitype_1.parseAbi)(['function L1_NULLIFIER() view returns (address)']),
            functionName: 'L1_NULLIFIER',
            args: [],
        });
    if (!l1NativeTokenVault)
        l1NativeTokenVault = await (0, readContract_js_1.readContract)(client, {
            address: addresses.sharedL1,
            abi: (0, abitype_1.parseAbi)(['function nativeTokenVault() view returns (address)']),
            functionName: 'nativeTokenVault',
            args: [],
        });
    return {
        ...addresses,
        l1Nullifier,
        l1NativeTokenVault,
    };
}
function scaleGasLimit(gasLimit) {
    return (gasLimit * BigInt(12)) / BigInt(10);
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
//# sourceMappingURL=deposit.js.map