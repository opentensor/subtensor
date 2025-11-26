"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.from = from;
exports.fromRpc = fromRpc;
exports.getSignPayload = getSignPayload;
exports.hash = hash;
exports.toPacked = toPacked;
exports.toRpc = toRpc;
const AbiParameters = require("../core/AbiParameters.js");
const Hash = require("../core/Hash.js");
const Hex = require("../core/Hex.js");
const Signature = require("../core/Signature.js");
function from(userOperation, options = {}) {
    const signature = (() => {
        if (!options.signature)
            return undefined;
        if (typeof options.signature === 'string')
            return options.signature;
        return Signature.toHex(options.signature);
    })();
    return { ...userOperation, signature };
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
function toPacked(userOperation) {
    const { callGasLimit, callData, factory, factoryData, maxPriorityFeePerGas, maxFeePerGas, nonce, paymaster, paymasterData, paymasterPostOpGasLimit, paymasterVerificationGasLimit, sender, signature, verificationGasLimit, } = userOperation;
    const accountGasLimits = Hex.concat(Hex.padLeft(Hex.fromNumber(verificationGasLimit || 0n), 16), Hex.padLeft(Hex.fromNumber(callGasLimit || 0n), 16));
    const initCode = factory && factoryData ? Hex.concat(factory, factoryData) : '0x';
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
//# sourceMappingURL=UserOperation.js.map