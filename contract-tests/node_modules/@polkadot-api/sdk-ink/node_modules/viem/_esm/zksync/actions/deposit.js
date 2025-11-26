import { parseAbi } from 'abitype';
import { estimateGas, } from '../../actions/public/estimateGas.js';
import { readContract } from '../../actions/public/readContract.js';
import { waitForTransactionReceipt } from '../../actions/public/waitForTransactionReceipt.js';
import { sendTransaction, } from '../../actions/wallet/sendTransaction.js';
import { writeContract, } from '../../actions/wallet/writeContract.js';
import { publicActions } from '../../clients/decorators/public.js';
import { erc20Abi } from '../../constants/abis.js';
import { zeroAddress } from '../../constants/address.js';
import { zeroHash } from '../../constants/bytes.js';
import { AccountNotFoundError } from '../../errors/account.js';
import { ClientChainNotConfiguredError } from '../../errors/chain.js';
import { concatHex, encodeAbiParameters, encodeFunctionData, isAddressEqual, keccak256, parseAccount, } from '../../utils/index.js';
import { bridgehubAbi } from '../constants/abis.js';
import { ethAddressInContracts, l2NativeTokenVaultAddress, legacyEthAddress, } from '../constants/address.js';
import { requiredL1ToL2GasPerPubdataLimit } from '../constants/number.js';
import { BaseFeeHigherThanValueError, } from '../errors/bridge.js';
import { applyL1ToL2Alias } from '../utils/bridge/applyL1ToL2Alias.js';
import { estimateGasL1ToL2 } from './estimateGasL1ToL2.js';
import { getBridgehubContractAddress } from './getBridgehubContractAddress.js';
import { getDefaultBridgeAddresses } from './getDefaultBridgeAddresses.js';
import { getL1Allowance } from './getL1Allowance.js';
/**
 * Transfers the specified token from the associated account on the L1 network to the target account on the L2 network.
 * The token can be either ETH or any ERC20 token. For ERC20 tokens, enough approved tokens must be associated with
 * the specified L1 bridge (default one or the one defined in `bridgeAddress`).
 * In this case, depending on is the chain ETH-based or not `approveToken` or `approveBaseToken`
 * can be enabled to perform token approval. If there are already enough approved tokens for the L1 bridge,
 * token approval will be skipped.
 *
 * @param client - Client to use
 * @param parameters - {@link DepositParameters}
 * @returns hash - The [Transaction](https://viem.sh/docs/glossary/terms#transaction) hash. {@link DepositReturnType}
 *
 * @example
 * import { createPublicClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { zksync, mainnet } from 'viem/chains'
 * import { deposit, legacyEthAddress, publicActionsL2 } from 'viem/zksync'
 *
 * const client = createPublicClient({
 *     chain: mainnet,
 *     transport: http(),
 * })
 *
 * const clientL2 = createPublicClient({
 *   chain: zksync,
 *   transport: http(),
 * }).extend(publicActionsL2())
 *
 * const account = privateKeyToAccount('0x…')
 *
 * const hash = await deposit(client, {
 *     client: clientL2,
 *     account,
 *     token: legacyEthAddress,
 *     to: account.address,
 *     amount: 1_000_000_000_000_000_000n,
 *     refundRecipient: account.address,
 * })
 *
 * @example Account Hoisting
 * import { createPublicClient, createWalletClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { zksync, mainnet } from 'viem/chains'
 * import { legacyEthAddress, publicActionsL2 } from 'viem/zksync'
 *
 * const walletClient = createWalletClient({
 *   chain: mainnet,
 *   transport: http(),
 *   account: privateKeyToAccount('0x…'),
 * })
 *
 * const clientL2 = createPublicClient({
 *   chain: zksync,
 *   transport: http(),
 * }).extend(publicActionsL2())
 *
 * const hash = await deposit(walletClient, {
 *     client: clientL2,
 *     token: legacyEthAddress,
 *     to: walletClient.account.address,
 *     amount: 1_000_000_000_000_000_000n,
 *     refundRecipient: walletClient.account.address,
 * })
 */
export async function deposit(client, parameters) {
    let { account: account_ = client.account, chain: chain_ = client.chain, client: l2Client, token, amount, approveToken, approveBaseToken, gas, } = parameters;
    const account = account_ ? parseAccount(account_) : client.account;
    if (!account)
        throw new AccountNotFoundError({
            docsPath: '/docs/actions/wallet/sendTransaction',
        });
    if (!l2Client.chain)
        throw new ClientChainNotConfiguredError();
    if (isAddressEqual(token, legacyEthAddress))
        token = ethAddressInContracts;
    const bridgeAddresses = await getBridgeAddresses(client, l2Client);
    const bridgehub = await getBridgehubContractAddress(l2Client);
    const baseToken = await readContract(client, {
        address: bridgehub,
        abi: bridgehubAbi,
        functionName: 'baseToken',
        args: [BigInt(l2Client.chain.id)],
    });
    const { mintValue, tx } = await getL1DepositTx(client, account, { ...parameters, token }, bridgeAddresses, bridgehub, baseToken);
    await approveTokens(client, chain_, tx.bridgeAddress, baseToken, mintValue, account, token, amount, approveToken, approveBaseToken);
    if (!gas) {
        const baseGasLimit = await estimateGas(client, {
            account: account.address,
            to: bridgehub,
            value: tx.value,
            data: tx.data,
        });
        gas = scaleGasLimit(baseGasLimit);
    }
    return await sendTransaction(client, {
        chain: chain_,
        account,
        gas,
        ...tx,
    });
}
async function getL1DepositTx(client, account, parameters, bridgeAddresses, bridgehub, baseToken) {
    let { account: _account, chain: _chain, client: l2Client, token, amount, to, operatorTip = 0n, l2GasLimit, gasPerPubdataByte = requiredL1ToL2GasPerPubdataLimit, refundRecipient = zeroAddress, bridgeAddress, customBridgeData, value, gasPrice, maxFeePerGas, maxPriorityFeePerGas, approveToken: _approveToken, approveBaseToken: _approveBaseToken, ...rest } = parameters;
    if (!l2Client.chain)
        throw new ClientChainNotConfiguredError();
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
    const isETHBasedChain = isAddressEqual(baseToken, ethAddressInContracts);
    if ((isETHBasedChain && isAddressEqual(token, ethAddressInContracts)) || // ETH on ETH-based chain
        isAddressEqual(token, baseToken) // base token on custom chain
    ) {
        // Deposit base token
        mintValue = baseCost + operatorTip + amount;
        let providedValue = isETHBasedChain ? value : mintValue;
        if (!providedValue || providedValue === 0n)
            providedValue = mintValue;
        if (baseCost > providedValue)
            throw new BaseFeeHigherThanValueError(baseCost, providedValue);
        value = isETHBasedChain ? providedValue : 0n;
        bridgeAddress = bridgeAddresses.sharedL1; // required for approval of base token on custom chain
        data = encodeFunctionData({
            abi: bridgehubAbi,
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
    else if (isAddressEqual(baseToken, ethAddressInContracts)) {
        // Deposit token on ETH-based chain
        mintValue = baseCost + BigInt(operatorTip);
        value = mintValue;
        if (baseCost > mintValue)
            throw new BaseFeeHigherThanValueError(baseCost, mintValue);
        bridgeAddress ??= bridgeAddresses.sharedL1;
        data = encodeFunctionData({
            abi: bridgehubAbi,
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
    else if (isAddressEqual(token, ethAddressInContracts)) {
        // Deposit ETH on custom chain
        mintValue = baseCost + operatorTip;
        value = amount;
        if (baseCost > mintValue)
            throw new BaseFeeHigherThanValueError(baseCost, mintValue);
        bridgeAddress = bridgeAddresses.sharedL1;
        data = encodeFunctionData({
            abi: bridgehubAbi,
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
                    secondBridgeCalldata: await getSecondBridgeCalldata(client, bridgeAddresses.l1NativeTokenVault, ethAddressInContracts, amount, to),
                },
            ],
        });
    }
    else {
        // Deposit token on custom chain
        mintValue = baseCost + operatorTip;
        value ??= 0n;
        if (baseCost > mintValue)
            throw new BaseFeeHigherThanValueError(baseCost, mintValue);
        bridgeAddress ??= bridgeAddresses.sharedL1;
        data = encodeFunctionData({
            abi: bridgehubAbi,
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
    if (isAddressEqual(baseToken, ethAddressInContracts)) {
        // Deposit token on ETH-based chain
        if (approveToken) {
            const overrides = typeof approveToken === 'boolean' ? {} : approveToken;
            const allowance = await getL1Allowance(client, {
                token,
                bridgeAddress,
                account,
            });
            if (allowance < amount) {
                const hash = await writeContract(client, {
                    chain,
                    account,
                    address: token,
                    abi: erc20Abi,
                    functionName: 'approve',
                    args: [bridgeAddress, amount],
                    ...overrides,
                });
                await waitForTransactionReceipt(client, { hash });
            }
        }
        return;
    }
    if (isAddressEqual(token, ethAddressInContracts)) {
        // Deposit ETH on custom chain
        if (approveBaseToken) {
            const overrides = typeof approveToken === 'boolean' ? {} : approveToken;
            const allowance = await getL1Allowance(client, {
                token: baseToken,
                bridgeAddress,
                account,
            });
            if (allowance < mintValue) {
                const hash = await writeContract(client, {
                    chain,
                    account,
                    address: baseToken,
                    abi: erc20Abi,
                    functionName: 'approve',
                    args: [bridgeAddress, mintValue],
                    ...overrides,
                });
                await waitForTransactionReceipt(client, { hash });
            }
            return;
        }
    }
    if (isAddressEqual(token, baseToken)) {
        // Deposit base token on custom chain
        if (approveToken || approveBaseToken) {
            const overrides = typeof approveToken === 'boolean'
                ? {}
                : (approveToken ?? typeof approveBaseToken === 'boolean')
                    ? {}
                    : approveBaseToken;
            const allowance = await getL1Allowance(client, {
                token: baseToken,
                bridgeAddress,
                account,
            });
            if (allowance < mintValue) {
                const hash = await writeContract(client, {
                    chain,
                    account,
                    address: baseToken,
                    abi: erc20Abi,
                    functionName: 'approve',
                    args: [bridgeAddress, mintValue],
                    ...overrides,
                });
                await waitForTransactionReceipt(client, { hash });
            }
        }
        return;
    }
    // Deposit token on custom chain
    if (approveBaseToken) {
        const overrides = typeof approveToken === 'boolean' ? {} : approveToken;
        const allowance = await getL1Allowance(client, {
            token: baseToken,
            bridgeAddress,
            account,
        });
        if (allowance < mintValue) {
            const hash = await writeContract(client, {
                chain,
                account,
                address: baseToken,
                abi: erc20Abi,
                functionName: 'approve',
                args: [bridgeAddress, mintValue],
                ...overrides,
            });
            await waitForTransactionReceipt(client, { hash });
        }
    }
    if (approveToken) {
        const overrides = typeof approveToken === 'boolean' ? {} : approveToken;
        const allowance = await getL1Allowance(client, {
            token,
            bridgeAddress,
            account,
        });
        if (allowance < amount) {
            const hash = await writeContract(client, {
                chain,
                account,
                address: token,
                abi: erc20Abi,
                functionName: 'approve',
                args: [bridgeAddress, amount],
                ...overrides,
            });
            await waitForTransactionReceipt(client, { hash });
        }
    }
}
async function getL2BridgeTxFeeParams(client, l2Client, bridgehub, gasPrice, from, token, amount, to, gasPerPubdataByte, baseToken, l2GasLimit, bridgeAddress, customBridgeData) {
    if (!l2Client.chain)
        throw new ClientChainNotConfiguredError();
    let l2GasLimit_ = l2GasLimit;
    if (!l2GasLimit_)
        l2GasLimit_ = bridgeAddress
            ? await getL2GasLimitFromCustomBridge(client, l2Client, from, token, amount, to, gasPerPubdataByte, bridgeAddress, customBridgeData)
            : await getL2GasLimitFromDefaultBridge(client, l2Client, from, token, amount, to, gasPerPubdataByte, baseToken);
    const baseCost = await readContract(client, {
        address: bridgehub,
        abi: bridgehubAbi,
        functionName: 'l2TransactionBaseCost',
        args: [BigInt(l2Client.chain.id), gasPrice, l2GasLimit_, gasPerPubdataByte],
    });
    return { l2GasLimit_, baseCost };
}
async function getL2GasLimitFromDefaultBridge(client, l2Client, from, token, amount, to, gasPerPubdataByte, baseToken) {
    if (isAddressEqual(token, baseToken)) {
        return await estimateGasL1ToL2(l2Client, {
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
    const bridgeAddresses = await getDefaultBridgeAddresses(l2Client);
    const l1BridgeAddress = bridgeAddresses.sharedL1;
    const l2BridgeAddress = bridgeAddresses.sharedL2;
    const bridgeData = await encodeDefaultBridgeData(client, token);
    const calldata = encodeFunctionData({
        abi: parseAbi([
            'function finalizeDeposit(address _l1Sender, address _l2Receiver, address _l1Token, uint256 _amount, bytes _data)',
        ]),
        functionName: 'finalizeDeposit',
        args: [
            from,
            to,
            isAddressEqual(token, legacyEthAddress) ? ethAddressInContracts : token,
            amount,
            bridgeData,
        ],
    });
    return await estimateGasL1ToL2(l2Client, {
        chain: l2Client.chain,
        account: applyL1ToL2Alias(l1BridgeAddress),
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
    const l2BridgeAddress = await readContract(client, {
        address: token,
        abi: parseAbi([
            'function l2BridgeAddress(uint256 _chainId) view returns (address)',
        ]),
        functionName: 'l2BridgeAddress',
        args: [BigInt(l2Client.chain.id)],
    });
    const calldata = encodeFunctionData({
        abi: parseAbi([
            'function finalizeDeposit(address _l1Sender, address _l2Receiver, address _l1Token, uint256 _amount, bytes _data)',
        ]),
        functionName: 'finalizeDeposit',
        args: [from, to, token, amount, customBridgeData_],
    });
    return await estimateGasL1ToL2(l2Client, {
        chain: l2Client.chain,
        account: from,
        from: applyL1ToL2Alias(bridgeAddress),
        to: l2BridgeAddress,
        data: calldata,
        gasPerPubdata: gasPerPubdataByte,
    });
}
async function encodeDefaultBridgeData(client, token) {
    let token_ = token;
    if (isAddressEqual(token, legacyEthAddress))
        token_ = ethAddressInContracts;
    let name = 'Ether';
    let symbol = 'ETH';
    let decimals = 18n;
    if (!isAddressEqual(token_, ethAddressInContracts)) {
        name = await readContract(client, {
            address: token_,
            abi: erc20Abi,
            functionName: 'name',
            args: [],
        });
        symbol = await readContract(client, {
            address: token_,
            abi: erc20Abi,
            functionName: 'symbol',
            args: [],
        });
        decimals = BigInt(await readContract(client, {
            address: token_,
            abi: erc20Abi,
            functionName: 'decimals',
            args: [],
        }));
    }
    const nameBytes = encodeAbiParameters([{ type: 'string' }], [name]);
    const symbolBytes = encodeAbiParameters([{ type: 'string' }], [symbol]);
    const decimalsBytes = encodeAbiParameters([{ type: 'uint256' }], [decimals]);
    return encodeAbiParameters([{ type: 'bytes' }, { type: 'bytes' }, { type: 'bytes' }], [nameBytes, symbolBytes, decimalsBytes]);
}
async function getSecondBridgeCalldata(client, l1NativeTokenVault, token, amount, to) {
    let assetId;
    let token_ = token;
    if (isAddressEqual(token, legacyEthAddress))
        token_ = ethAddressInContracts;
    const assetIdFromNTV = await readContract(client, {
        address: l1NativeTokenVault,
        abi: parseAbi(['function assetId(address token) view returns (bytes32)']),
        functionName: 'assetId',
        args: [token_],
    });
    if (assetIdFromNTV && assetIdFromNTV !== zeroHash)
        assetId = assetIdFromNTV;
    else {
        // Okay, the token have not been registered within the Native token vault.
        // There are two cases when it is possible:
        // - The token is native to L1 (it may or may not be bridged), but it has not been
        // registered within NTV after the Gateway upgrade. We assume that this is not the case
        // as the SDK is expected to work only after the full migration is done.
        // - The token is native to the current chain and it has never been bridged.
        if (!client.chain)
            throw new ClientChainNotConfiguredError();
        assetId = keccak256(encodeAbiParameters([{ type: 'uint256' }, { type: 'address' }, { type: 'address' }], [BigInt(client.chain.id), l2NativeTokenVaultAddress, token_]));
    }
    const ntvData = encodeAbiParameters([{ type: 'uint256' }, { type: 'address' }, { type: 'address' }], [BigInt(amount), to, token_]);
    const data = encodeAbiParameters([{ type: 'bytes32' }, { type: 'bytes' }], [assetId, ntvData]);
    return concatHex(['0x01', data]);
}
async function getBridgeAddresses(client, l2Client) {
    const addresses = await getDefaultBridgeAddresses(l2Client);
    let l1Nullifier = addresses.l1Nullifier;
    let l1NativeTokenVault = addresses.l1NativeTokenVault;
    if (!l1Nullifier)
        l1Nullifier = await readContract(client, {
            address: addresses.sharedL1,
            abi: parseAbi(['function L1_NULLIFIER() view returns (address)']),
            functionName: 'L1_NULLIFIER',
            args: [],
        });
    if (!l1NativeTokenVault)
        l1NativeTokenVault = await readContract(client, {
            address: addresses.sharedL1,
            abi: parseAbi(['function nativeTokenVault() view returns (address)']),
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
    const client_ = client.extend(publicActions);
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