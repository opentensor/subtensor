"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.from = from;
exports.fromRpc = fromRpc;
exports.getSignPayload = getSignPayload;
exports.hash = hash;
exports.toInitCode = toInitCode;
exports.toPacked = toPacked;
exports.fromPacked = fromPacked;
exports.toRpc = toRpc;
exports.toTypedData = toTypedData;
const AbiParameters = require("../core/AbiParameters.js");
const Hash = require("../core/Hash.js");
const Hex = require("../core/Hex.js");
const Signature = require("../core/Signature.js");
const TypedData = require("../core/TypedData.js");
function from(userOperation, options = {}) {
    const signature = (() => {
        if (typeof options.signature === 'string')
            return options.signature;
        if (typeof options.signature === 'object')
            return Signature.toHex(options.signature);
        if (userOperation.signature)
            return userOperation.signature;
        return undefined;
    })();
    const packed = 'accountGasLimits' in userOperation && 'gasFees' in userOperation;
    const userOp = packed ? fromPacked(userOperation) : userOperation;
    return { ...userOp, signature };
}
function fromRpc(rpc) {
    return {
        ...rpc,
        callGasLimit: BigInt(rpc.callGasLimit),
        maxFeePerGas: BigInt(rpc.maxFeePerGas),
        maxPriorityFeePerGas: BigInt(rpc.maxPriorityFeePerGas),
        nonce: BigInt(rpc.nonce),
        preVerificationGas: BigInt(rpc.preVerificationGas),
        verificationGasLimit: BigInt(rpc.verificationGasLimit),
        ...(rpc.paymasterPostOpGasLimit && {
            paymasterPostOpGasLimit: BigInt(rpc.paymasterPostOpGasLimit),
        }),
        ...(rpc.paymasterVerificationGasLimit && {
            paymasterVerificationGasLimit: BigInt(rpc.paymasterVerificationGasLimit),
        }),
    };
}
function getSignPayload(userOperation, options) {
    return hash(userOperation, options);
}
function hash(userOperation, options) {
    const { chainId, entryPointAddress, entryPointVersion } = options;
    const { callData, callGasLimit, initCode, factory, factoryData, maxFeePerGas, maxPriorityFeePerGas, nonce, paymaster, paymasterAndData, paymasterData, paymasterPostOpGasLimit, paymasterVerificationGasLimit, preVerificationGas, sender, verificationGasLimit, } = userOperation;
    if (entryPointVersion === '0.8') {
        const typedData = toTypedData(userOperation, {
            chainId,
            entryPointAddress,
        });
        return TypedData.getSignPayload(typedData);
    }
    const packedUserOp = (() => {
        if (entryPointVersion === '0.6') {
            return AbiParameters.encode([
                { type: 'address' },
                { type: 'uint256' },
                { type: 'bytes32' },
                { type: 'bytes32' },
                { type: 'uint256' },
                { type: 'uint256' },
                { type: 'uint256' },
                { type: 'uint256' },
                { type: 'uint256' },
                { type: 'bytes32' },
            ], [
                sender,
                nonce,
                Hash.keccak256(initCode ?? '0x'),
                Hash.keccak256(callData),
                callGasLimit,
                verificationGasLimit,
                preVerificationGas,
                maxFeePerGas,
                maxPriorityFeePerGas,
                Hash.keccak256(paymasterAndData ?? '0x'),
            ]);
        }
        if (entryPointVersion === '0.7') {
            const accountGasLimits = Hex.concat(Hex.padLeft(Hex.fromNumber(verificationGasLimit), 16), Hex.padLeft(Hex.fromNumber(callGasLimit), 16));
            const gasFees = Hex.concat(Hex.padLeft(Hex.fromNumber(maxPriorityFeePerGas), 16), Hex.padLeft(Hex.fromNumber(maxFeePerGas), 16));
            const initCode_hashed = Hash.keccak256(factory && factoryData ? Hex.concat(factory, factoryData) : '0x');
            const paymasterAndData_hashed = Hash.keccak256(paymaster
                ? Hex.concat(paymaster, Hex.padLeft(Hex.fromNumber(paymasterVerificationGasLimit || 0), 16), Hex.padLeft(Hex.fromNumber(paymasterPostOpGasLimit || 0), 16), paymasterData || '0x')
                : '0x');
            return AbiParameters.encode([
                { type: 'address' },
                { type: 'uint256' },
                { type: 'bytes32' },
                { type: 'bytes32' },
                { type: 'bytes32' },
                { type: 'uint256' },
                { type: 'bytes32' },
                { type: 'bytes32' },
            ], [
                sender,
                nonce,
                initCode_hashed,
                Hash.keccak256(callData),
                accountGasLimits,
                preVerificationGas,
                gasFees,
                paymasterAndData_hashed,
            ]);
        }
        throw new Error(`entryPointVersion "${entryPointVersion}" not supported.`);
    })();
    return Hash.keccak256(AbiParameters.encode([{ type: 'bytes32' }, { type: 'address' }, { type: 'uint256' }], [Hash.keccak256(packedUserOp), entryPointAddress, BigInt(chainId)]));
}
function toInitCode(userOperation) {
    const { authorization, factory, factoryData } = userOperation;
    if (factory === '0x7702' ||
        factory === '0x7702000000000000000000000000000000000000') {
        if (!authorization)
            return '0x7702000000000000000000000000000000000000';
        const delegation = authorization.address;
        return Hex.concat(delegation, factoryData ?? '0x');
    }
    if (!factory)
        return '0x';
    return Hex.concat(factory, factoryData ?? '0x');
}
function toPacked(userOperation) {
    const { callGasLimit, callData, maxPriorityFeePerGas, maxFeePerGas, nonce, paymaster, paymasterData, paymasterPostOpGasLimit, paymasterVerificationGasLimit, sender, signature, verificationGasLimit, } = userOperation;
    const accountGasLimits = Hex.concat(Hex.padLeft(Hex.fromNumber(verificationGasLimit || 0n), 16), Hex.padLeft(Hex.fromNumber(callGasLimit || 0n), 16));
    const initCode = toInitCode(userOperation);
    const gasFees = Hex.concat(Hex.padLeft(Hex.fromNumber(maxPriorityFeePerGas || 0n), 16), Hex.padLeft(Hex.fromNumber(maxFeePerGas || 0n), 16));
    const paymasterAndData = paymaster
        ? Hex.concat(paymaster, Hex.padLeft(Hex.fromNumber(paymasterVerificationGasLimit || 0n), 16), Hex.padLeft(Hex.fromNumber(paymasterPostOpGasLimit || 0n), 16), paymasterData || '0x')
        : '0x';
    const preVerificationGas = userOperation.preVerificationGas ?? 0n;
    return {
        accountGasLimits,
        callData,
        initCode,
        gasFees,
        nonce,
        paymasterAndData,
        preVerificationGas,
        sender,
        signature,
    };
}
function fromPacked(packed) {
    const { accountGasLimits, callData, initCode, gasFees, nonce, paymasterAndData, preVerificationGas, sender, signature, } = packed;
    const verificationGasLimit = BigInt(Hex.slice(accountGasLimits, 0, 16));
    const callGasLimit = BigInt(Hex.slice(accountGasLimits, 16, 32));
    const { factory, factoryData } = (() => {
        if (initCode === '0x')
            return { factory: undefined, factoryData: undefined };
        const factory = Hex.slice(initCode, 0, 20);
        const factoryData = Hex.size(initCode) > 20 ? Hex.slice(initCode, 20) : undefined;
        return { factory, factoryData };
    })();
    const maxPriorityFeePerGas = BigInt(Hex.slice(gasFees, 0, 16));
    const maxFeePerGas = BigInt(Hex.slice(gasFees, 16, 32));
    const { paymaster, paymasterVerificationGasLimit, paymasterPostOpGasLimit, paymasterData, } = (() => {
        if (paymasterAndData === '0x')
            return {
                paymaster: undefined,
                paymasterVerificationGasLimit: undefined,
                paymasterPostOpGasLimit: undefined,
                paymasterData: undefined,
            };
        const paymaster = Hex.slice(paymasterAndData, 0, 20);
        const paymasterVerificationGasLimit = BigInt(Hex.slice(paymasterAndData, 20, 36));
        const paymasterPostOpGasLimit = BigInt(Hex.slice(paymasterAndData, 36, 52));
        const paymasterData = Hex.size(paymasterAndData) > 52
            ? Hex.slice(paymasterAndData, 52)
            : undefined;
        return {
            paymaster,
            paymasterVerificationGasLimit,
            paymasterPostOpGasLimit,
            paymasterData,
        };
    })();
    return {
        callData,
        callGasLimit,
        ...(factory && { factory }),
        ...(factoryData && { factoryData }),
        maxFeePerGas,
        maxPriorityFeePerGas,
        nonce,
        ...(paymaster && { paymaster }),
        ...(paymasterData && { paymasterData }),
        ...(typeof paymasterPostOpGasLimit === 'bigint' && {
            paymasterPostOpGasLimit,
        }),
        ...(typeof paymasterVerificationGasLimit === 'bigint' && {
            paymasterVerificationGasLimit,
        }),
        preVerificationGas,
        sender,
        signature,
        verificationGasLimit,
    };
}
function toRpc(userOperation) {
    const rpc = {};
    rpc.callData = userOperation.callData;
    rpc.callGasLimit = Hex.fromNumber(userOperation.callGasLimit);
    rpc.maxFeePerGas = Hex.fromNumber(userOperation.maxFeePerGas);
    rpc.maxPriorityFeePerGas = Hex.fromNumber(userOperation.maxPriorityFeePerGas);
    rpc.nonce = Hex.fromNumber(userOperation.nonce);
    rpc.preVerificationGas = Hex.fromNumber(userOperation.preVerificationGas);
    rpc.sender = userOperation.sender;
    rpc.verificationGasLimit = Hex.fromNumber(userOperation.verificationGasLimit);
    if (userOperation.factory)
        rpc.factory = userOperation.factory;
    if (userOperation.factoryData)
        rpc.factoryData = userOperation.factoryData;
    if (userOperation.initCode)
        rpc.initCode = userOperation.initCode;
    if (userOperation.paymaster)
        rpc.paymaster = userOperation.paymaster;
    if (userOperation.paymasterData)
        rpc.paymasterData = userOperation.paymasterData;
    if (typeof userOperation.paymasterPostOpGasLimit === 'bigint')
        rpc.paymasterPostOpGasLimit = Hex.fromNumber(userOperation.paymasterPostOpGasLimit);
    if (typeof userOperation.paymasterVerificationGasLimit === 'bigint')
        rpc.paymasterVerificationGasLimit = Hex.fromNumber(userOperation.paymasterVerificationGasLimit);
    if (userOperation.signature)
        rpc.signature = userOperation.signature;
    return rpc;
}
function toTypedData(userOperation, options) {
    const { chainId, entryPointAddress } = options;
    const packedUserOp = toPacked(userOperation);
    return {
        domain: {
            name: 'ERC4337',
            version: '1',
            chainId,
            verifyingContract: entryPointAddress,
        },
        message: packedUserOp,
        primaryType: 'PackedUserOperation',
        types: toTypedData.types,
    };
}
(function (toTypedData) {
    toTypedData.types = {
        PackedUserOperation: [
            { type: 'address', name: 'sender' },
            { type: 'uint256', name: 'nonce' },
            { type: 'bytes', name: 'initCode' },
            { type: 'bytes', name: 'callData' },
            { type: 'bytes32', name: 'accountGasLimits' },
            { type: 'uint256', name: 'preVerificationGas' },
            { type: 'bytes32', name: 'gasFees' },
            { type: 'bytes', name: 'paymasterAndData' },
        ],
    };
})(toTypedData || (exports.toTypedData = toTypedData = {}));
//# sourceMappingURL=UserOperation.js.map