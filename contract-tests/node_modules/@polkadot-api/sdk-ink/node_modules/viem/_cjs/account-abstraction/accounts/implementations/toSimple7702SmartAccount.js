"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.toSimple7702SmartAccount = toSimple7702SmartAccount;
const address_js_1 = require("../../../constants/address.js");
const base_js_1 = require("../../../errors/base.js");
const decodeFunctionData_js_1 = require("../../../utils/abi/decodeFunctionData.js");
const encodeFunctionData_js_1 = require("../../../utils/abi/encodeFunctionData.js");
const abis_js_1 = require("../../constants/abis.js");
const getUserOperationTypedData_js_1 = require("../../utils/userOperation/getUserOperationTypedData.js");
const toSmartAccount_js_1 = require("../toSmartAccount.js");
async function toSimple7702SmartAccount(parameters) {
    const { client, implementation = '0xe6Cae83BdE06E4c305530e199D7217f42808555B', getNonce, owner, } = parameters;
    const entryPoint = {
        abi: abis_js_1.entryPoint08Abi,
        address: address_js_1.entryPoint08Address,
        version: '0.8',
    };
    return (0, toSmartAccount_js_1.toSmartAccount)({
        authorization: { account: owner, address: implementation },
        abi,
        client,
        extend: { abi, owner },
        entryPoint,
        getNonce,
        async decodeCalls(data) {
            const result = (0, decodeFunctionData_js_1.decodeFunctionData)({
                abi,
                data,
            });
            if (result.functionName === 'execute')
                return [
                    { to: result.args[0], value: result.args[1], data: result.args[2] },
                ];
            if (result.functionName === 'executeBatch')
                return result.args[0].map((arg) => ({
                    to: arg.target,
                    value: arg.value,
                    data: arg.data,
                }));
            throw new base_js_1.BaseError(`unable to decode calls for "${result.functionName}"`);
        },
        async encodeCalls(calls) {
            if (calls.length === 1)
                return (0, encodeFunctionData_js_1.encodeFunctionData)({
                    abi,
                    functionName: 'execute',
                    args: [calls[0].to, calls[0].value ?? 0n, calls[0].data ?? '0x'],
                });
            return (0, encodeFunctionData_js_1.encodeFunctionData)({
                abi,
                functionName: 'executeBatch',
                args: [
                    calls.map((call) => ({
                        data: call.data ?? '0x',
                        target: call.to,
                        value: call.value ?? 0n,
                    })),
                ],
            });
        },
        async getAddress() {
            return owner.address;
        },
        async getFactoryArgs() {
            return { factory: '0x7702', factoryData: '0x' };
        },
        async getStubSignature() {
            return '0xfffffffffffffffffffffffffffffff0000000000000000000000000000000007aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1c';
        },
        async signMessage(parameters) {
            const { message } = parameters;
            return await owner.signMessage({ message });
        },
        async signTypedData(parameters) {
            const { domain, types, primaryType, message } = parameters;
            return await owner.signTypedData({
                domain,
                message,
                primaryType,
                types,
            });
        },
        async signUserOperation(parameters) {
            const { chainId = client.chain.id, ...userOperation } = parameters;
            const address = await this.getAddress();
            const typedData = (0, getUserOperationTypedData_js_1.getUserOperationTypedData)({
                chainId,
                entryPointAddress: entryPoint.address,
                userOperation: {
                    ...userOperation,
                    sender: address,
                },
            });
            return await owner.signTypedData(typedData);
        },
    });
}
const abi = [
    { inputs: [], name: 'ECDSAInvalidSignature', type: 'error' },
    {
        inputs: [{ internalType: 'uint256', name: 'length', type: 'uint256' }],
        name: 'ECDSAInvalidSignatureLength',
        type: 'error',
    },
    {
        inputs: [{ internalType: 'bytes32', name: 's', type: 'bytes32' }],
        name: 'ECDSAInvalidSignatureS',
        type: 'error',
    },
    {
        inputs: [
            { internalType: 'uint256', name: 'index', type: 'uint256' },
            { internalType: 'bytes', name: 'error', type: 'bytes' },
        ],
        name: 'ExecuteError',
        type: 'error',
    },
    { stateMutability: 'payable', type: 'fallback' },
    {
        inputs: [],
        name: 'entryPoint',
        outputs: [
            { internalType: 'contract IEntryPoint', name: '', type: 'address' },
        ],
        stateMutability: 'pure',
        type: 'function',
    },
    {
        inputs: [
            { internalType: 'address', name: 'target', type: 'address' },
            { internalType: 'uint256', name: 'value', type: 'uint256' },
            { internalType: 'bytes', name: 'data', type: 'bytes' },
        ],
        name: 'execute',
        outputs: [],
        stateMutability: 'nonpayable',
        type: 'function',
    },
    {
        inputs: [
            {
                components: [
                    { internalType: 'address', name: 'target', type: 'address' },
                    { internalType: 'uint256', name: 'value', type: 'uint256' },
                    { internalType: 'bytes', name: 'data', type: 'bytes' },
                ],
                internalType: 'struct BaseAccount.Call[]',
                name: 'calls',
                type: 'tuple[]',
            },
        ],
        name: 'executeBatch',
        outputs: [],
        stateMutability: 'nonpayable',
        type: 'function',
    },
    {
        inputs: [],
        name: 'getNonce',
        outputs: [{ internalType: 'uint256', name: '', type: 'uint256' }],
        stateMutability: 'view',
        type: 'function',
    },
    {
        inputs: [
            { internalType: 'bytes32', name: 'hash', type: 'bytes32' },
            { internalType: 'bytes', name: 'signature', type: 'bytes' },
        ],
        name: 'isValidSignature',
        outputs: [{ internalType: 'bytes4', name: 'magicValue', type: 'bytes4' }],
        stateMutability: 'view',
        type: 'function',
    },
    {
        inputs: [
            { internalType: 'address', name: '', type: 'address' },
            { internalType: 'address', name: '', type: 'address' },
            { internalType: 'uint256[]', name: '', type: 'uint256[]' },
            { internalType: 'uint256[]', name: '', type: 'uint256[]' },
            { internalType: 'bytes', name: '', type: 'bytes' },
        ],
        name: 'onERC1155BatchReceived',
        outputs: [{ internalType: 'bytes4', name: '', type: 'bytes4' }],
        stateMutability: 'nonpayable',
        type: 'function',
    },
    {
        inputs: [
            { internalType: 'address', name: '', type: 'address' },
            { internalType: 'address', name: '', type: 'address' },
            { internalType: 'uint256', name: '', type: 'uint256' },
            { internalType: 'uint256', name: '', type: 'uint256' },
            { internalType: 'bytes', name: '', type: 'bytes' },
        ],
        name: 'onERC1155Received',
        outputs: [{ internalType: 'bytes4', name: '', type: 'bytes4' }],
        stateMutability: 'nonpayable',
        type: 'function',
    },
    {
        inputs: [
            { internalType: 'address', name: '', type: 'address' },
            { internalType: 'address', name: '', type: 'address' },
            { internalType: 'uint256', name: '', type: 'uint256' },
            { internalType: 'bytes', name: '', type: 'bytes' },
        ],
        name: 'onERC721Received',
        outputs: [{ internalType: 'bytes4', name: '', type: 'bytes4' }],
        stateMutability: 'nonpayable',
        type: 'function',
    },
    {
        inputs: [{ internalType: 'bytes4', name: 'id', type: 'bytes4' }],
        name: 'supportsInterface',
        outputs: [{ internalType: 'bool', name: '', type: 'bool' }],
        stateMutability: 'pure',
        type: 'function',
    },
    {
        inputs: [
            {
                components: [
                    { internalType: 'address', name: 'sender', type: 'address' },
                    { internalType: 'uint256', name: 'nonce', type: 'uint256' },
                    { internalType: 'bytes', name: 'initCode', type: 'bytes' },
                    { internalType: 'bytes', name: 'callData', type: 'bytes' },
                    {
                        internalType: 'bytes32',
                        name: 'accountGasLimits',
                        type: 'bytes32',
                    },
                    {
                        internalType: 'uint256',
                        name: 'preVerificationGas',
                        type: 'uint256',
                    },
                    { internalType: 'bytes32', name: 'gasFees', type: 'bytes32' },
                    { internalType: 'bytes', name: 'paymasterAndData', type: 'bytes' },
                    { internalType: 'bytes', name: 'signature', type: 'bytes' },
                ],
                internalType: 'struct PackedUserOperation',
                name: 'userOp',
                type: 'tuple',
            },
            { internalType: 'bytes32', name: 'userOpHash', type: 'bytes32' },
            { internalType: 'uint256', name: 'missingAccountFunds', type: 'uint256' },
        ],
        name: 'validateUserOp',
        outputs: [
            { internalType: 'uint256', name: 'validationData', type: 'uint256' },
        ],
        stateMutability: 'nonpayable',
        type: 'function',
    },
    { stateMutability: 'payable', type: 'receive' },
];
//# sourceMappingURL=toSimple7702SmartAccount.js.map