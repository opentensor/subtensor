"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.simulateCalls = simulateCalls;
const AbiConstructor = require("ox/AbiConstructor");
const AbiFunction = require("ox/AbiFunction");
const parseAccount_js_1 = require("../../accounts/utils/parseAccount.js");
const address_js_1 = require("../../constants/address.js");
const contracts_js_1 = require("../../constants/contracts.js");
const base_js_1 = require("../../errors/base.js");
const encodeFunctionData_js_1 = require("../../utils/abi/encodeFunctionData.js");
const index_js_1 = require("../../utils/index.js");
const createAccessList_js_1 = require("./createAccessList.js");
const simulateBlocks_js_1 = require("./simulateBlocks.js");
const getBalanceCode = '0x6080604052348015600e575f80fd5b5061016d8061001c5f395ff3fe608060405234801561000f575f80fd5b5060043610610029575f3560e01c8063f8b2cb4f1461002d575b5f80fd5b610047600480360381019061004291906100db565b61005d565b604051610054919061011e565b60405180910390f35b5f8173ffffffffffffffffffffffffffffffffffffffff16319050919050565b5f80fd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f6100aa82610081565b9050919050565b6100ba816100a0565b81146100c4575f80fd5b50565b5f813590506100d5816100b1565b92915050565b5f602082840312156100f0576100ef61007d565b5b5f6100fd848285016100c7565b91505092915050565b5f819050919050565b61011881610106565b82525050565b5f6020820190506101315f83018461010f565b9291505056fea26469706673582212203b9fe929fe995c7cf9887f0bdba8a36dd78e8b73f149b17d2d9ad7cd09d2dc6264736f6c634300081a0033';
async function simulateCalls(client, parameters) {
    const { blockNumber, blockTag, calls, stateOverrides, traceAssetChanges, traceTransfers, validation, } = parameters;
    const account = parameters.account
        ? (0, parseAccount_js_1.parseAccount)(parameters.account)
        : undefined;
    if (traceAssetChanges && !account)
        throw new base_js_1.BaseError('`account` is required when `traceAssetChanges` is true');
    const getBalanceData = account
        ? AbiConstructor.encode(AbiConstructor.from('constructor(bytes, bytes)'), {
            bytecode: contracts_js_1.deploylessCallViaBytecodeBytecode,
            args: [
                getBalanceCode,
                AbiFunction.encodeData(AbiFunction.from('function getBalance(address)'), [account.address]),
            ],
        })
        : undefined;
    const assetAddresses = traceAssetChanges
        ? await Promise.all(parameters.calls.map(async (call) => {
            if (!call.data && !call.abi)
                return;
            const { accessList } = await (0, createAccessList_js_1.createAccessList)(client, {
                account: account.address,
                ...call,
                data: call.abi ? (0, encodeFunctionData_js_1.encodeFunctionData)(call) : call.data,
            });
            return accessList.map(({ address, storageKeys }) => storageKeys.length > 0 ? address : null);
        })).then((x) => x.flat().filter(Boolean))
        : [];
    const resultsStateOverrides = stateOverrides?.map((override) => {
        if (override.address === account?.address)
            return {
                ...override,
                nonce: 0,
            };
        return override;
    });
    const blocks = await (0, simulateBlocks_js_1.simulateBlocks)(client, {
        blockNumber,
        blockTag: blockTag,
        blocks: [
            ...(traceAssetChanges
                ? [
                    {
                        calls: [{ data: getBalanceData }],
                        stateOverrides,
                    },
                    {
                        calls: assetAddresses.map((address, i) => ({
                            abi: [
                                AbiFunction.from('function balanceOf(address) returns (uint256)'),
                            ],
                            functionName: 'balanceOf',
                            args: [account.address],
                            to: address,
                            from: address_js_1.zeroAddress,
                            nonce: i,
                        })),
                        stateOverrides: [
                            {
                                address: address_js_1.zeroAddress,
                                nonce: 0,
                            },
                        ],
                    },
                ]
                : []),
            {
                calls: [...calls, {}].map((call, index) => ({
                    ...call,
                    from: account?.address,
                    nonce: index,
                })),
                stateOverrides: resultsStateOverrides,
            },
            ...(traceAssetChanges
                ? [
                    {
                        calls: [{ data: getBalanceData }],
                    },
                    {
                        calls: assetAddresses.map((address, i) => ({
                            abi: [
                                AbiFunction.from('function balanceOf(address) returns (uint256)'),
                            ],
                            functionName: 'balanceOf',
                            args: [account.address],
                            to: address,
                            from: address_js_1.zeroAddress,
                            nonce: i,
                        })),
                        stateOverrides: [
                            {
                                address: address_js_1.zeroAddress,
                                nonce: 0,
                            },
                        ],
                    },
                    {
                        calls: assetAddresses.map((address, i) => ({
                            to: address,
                            abi: [
                                AbiFunction.from('function decimals() returns (uint256)'),
                            ],
                            functionName: 'decimals',
                            from: address_js_1.zeroAddress,
                            nonce: i,
                        })),
                        stateOverrides: [
                            {
                                address: address_js_1.zeroAddress,
                                nonce: 0,
                            },
                        ],
                    },
                    {
                        calls: assetAddresses.map((address, i) => ({
                            to: address,
                            abi: [
                                AbiFunction.from('function tokenURI(uint256) returns (string)'),
                            ],
                            functionName: 'tokenURI',
                            args: [0n],
                            from: address_js_1.zeroAddress,
                            nonce: i,
                        })),
                        stateOverrides: [
                            {
                                address: address_js_1.zeroAddress,
                                nonce: 0,
                            },
                        ],
                    },
                    {
                        calls: assetAddresses.map((address, i) => ({
                            to: address,
                            abi: [AbiFunction.from('function symbol() returns (string)')],
                            functionName: 'symbol',
                            from: address_js_1.zeroAddress,
                            nonce: i,
                        })),
                        stateOverrides: [
                            {
                                address: address_js_1.zeroAddress,
                                nonce: 0,
                            },
                        ],
                    },
                ]
                : []),
        ],
        traceTransfers,
        validation,
    });
    const block_results = traceAssetChanges ? blocks[2] : blocks[0];
    const [block_ethPre, block_assetsPre, , block_ethPost, block_assetsPost, block_decimals, block_tokenURI, block_symbols,] = traceAssetChanges ? blocks : [];
    const { calls: block_calls, ...block } = block_results;
    const results = block_calls.slice(0, -1) ?? [];
    const ethPre = block_ethPre?.calls ?? [];
    const assetsPre = block_assetsPre?.calls ?? [];
    const balancesPre = [...ethPre, ...assetsPre].map((call) => call.status === 'success' ? (0, index_js_1.hexToBigInt)(call.data) : null);
    const ethPost = block_ethPost?.calls ?? [];
    const assetsPost = block_assetsPost?.calls ?? [];
    const balancesPost = [...ethPost, ...assetsPost].map((call) => call.status === 'success' ? (0, index_js_1.hexToBigInt)(call.data) : null);
    const decimals = (block_decimals?.calls ?? []).map((x) => x.status === 'success' ? x.result : null);
    const symbols = (block_symbols?.calls ?? []).map((x) => x.status === 'success' ? x.result : null);
    const tokenURI = (block_tokenURI?.calls ?? []).map((x) => x.status === 'success' ? x.result : null);
    const changes = [];
    for (const [i, balancePost] of balancesPost.entries()) {
        const balancePre = balancesPre[i];
        if (typeof balancePost !== 'bigint')
            continue;
        if (typeof balancePre !== 'bigint')
            continue;
        const decimals_ = decimals[i - 1];
        const symbol_ = symbols[i - 1];
        const tokenURI_ = tokenURI[i - 1];
        const token = (() => {
            if (i === 0)
                return {
                    address: address_js_1.ethAddress,
                    decimals: 18,
                    symbol: 'ETH',
                };
            return {
                address: assetAddresses[i - 1],
                decimals: tokenURI_ || decimals_ ? Number(decimals_ ?? 1) : undefined,
                symbol: symbol_ ?? undefined,
            };
        })();
        if (changes.some((change) => change.token.address === token.address))
            continue;
        changes.push({
            token,
            value: {
                pre: balancePre,
                post: balancePost,
                diff: balancePost - balancePre,
            },
        });
    }
    return {
        assetChanges: changes,
        block,
        results,
    };
}
//# sourceMappingURL=simulateCalls.js.map