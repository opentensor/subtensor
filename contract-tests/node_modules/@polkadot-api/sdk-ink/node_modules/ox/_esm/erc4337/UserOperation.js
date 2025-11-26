import * as AbiParameters from '../core/AbiParameters.js';
import * as Hash from '../core/Hash.js';
import * as Hex from '../core/Hex.js';
import * as Signature from '../core/Signature.js';
import * as TypedData from '../core/TypedData.js';
/**
 * Instantiates a {@link ox#UserOperation.UserOperation} from a provided input.
 *
 * @example
 * ```ts twoslash
 * import { Value } from 'ox'
 * import { UserOperation } from 'ox/erc4337'
 *
 * const userOperation = UserOperation.from({
 *   callData: '0xdeadbeef',
 *   callGasLimit: 300_000n,
 *   maxFeePerGas: Value.fromGwei('20'),
 *   maxPriorityFeePerGas: Value.fromGwei('2'),
 *   nonce: 69n,
 *   preVerificationGas: 100_000n,
 *   sender: '0x9f1fdab6458c5fc642fa0f4c5af7473c46837357',
 *   verificationGasLimit: 100_000n,
 * })
 * ```
 *
 * @example
 * ### From Packed User Operation
 *
 * ```ts twoslash
 * import { UserOperation } from 'ox/erc4337'
 *
 * const packed: UserOperation.Packed = {
 *   accountGasLimits: '0x...',
 *   callData: '0xdeadbeef',
 *   initCode: '0x',
 *   gasFees: '0x...',
 *   nonce: 69n,
 *   paymasterAndData: '0x',
 *   preVerificationGas: 100_000n,
 *   sender: '0x9f1fdab6458c5fc642fa0f4c5af7473c46837357',
 *   signature: '0x',
 * }
 *
 * const userOperation = UserOperation.from(packed)
 * ```
 *
 * @example
 * ### Attaching Signatures
 *
 * ```ts twoslash
 * import { Secp256k1, Value } from 'ox'
 * import { UserOperation } from 'ox/erc4337'
 *
 * const userOperation = UserOperation.from({
 *   callData: '0xdeadbeef',
 *   callGasLimit: 300_000n,
 *   maxFeePerGas: Value.fromGwei('20'),
 *   maxPriorityFeePerGas: Value.fromGwei('2'),
 *   nonce: 69n,
 *   preVerificationGas: 100_000n,
 *   sender: '0x9f1fdab6458c5fc642fa0f4c5af7473c46837357',
 *   verificationGasLimit: 100_000n,
 * })
 *
 * const payload = UserOperation.getSignPayload(userOperation, {
 *   chainId: 1,
 *   entryPointAddress: '0x1234567890123456789012345678901234567890',
 *   entryPointVersion: '0.7',
 * })
 *
 * const signature = Secp256k1.sign({ payload, privateKey: '0x...' })
 *
 * const userOperation_signed = UserOperation.from(userOperation, { signature }) // [!code focus]
 * ```
 *
 * @param userOperation - The user operation to instantiate (structured or packed format).
 * @returns User Operation.
 */
export function from(userOperation, options = {}) {
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
/**
 * Converts an {@link ox#UserOperation.Rpc} to an {@link ox#UserOperation.UserOperation}.
 *
 * @example
 * ```ts twoslash
 * import { UserOperation } from 'ox/erc4337'
 *
 * const userOperation = UserOperation.fromRpc({
 *   callData: '0xdeadbeef',
 *   callGasLimit: '0x69420',
 *   maxFeePerGas: '0x2ca6ae494',
 *   maxPriorityFeePerGas: '0x41cc3c0',
 *   nonce: '0x357',
 *   preVerificationGas: '0x69420',
 *   signature: '0x',
 *   sender: '0x1234567890123456789012345678901234567890',
 *   verificationGasLimit: '0x69420',
 * })
 * ```
 *
 * @param rpc - The RPC user operation to convert.
 * @returns An instantiated {@link ox#UserOperation.UserOperation}.
 */
export function fromRpc(rpc) {
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
/**
 * Obtains the signing payload for a {@link ox#UserOperation.UserOperation}.
 *
 * @example
 * ```ts twoslash
 * import { Secp256k1, Value } from 'ox'
 * import { UserOperation } from 'ox/erc4337'
 *
 * const userOperation = UserOperation.from({
 *   callData: '0xdeadbeef',
 *   callGasLimit: 300_000n,
 *   maxFeePerGas: Value.fromGwei('20'),
 *   maxPriorityFeePerGas: Value.fromGwei('2'),
 *   nonce: 69n,
 *   preVerificationGas: 100_000n,
 *   sender: '0x9f1fdab6458c5fc642fa0f4c5af7473c46837357',
 *   verificationGasLimit: 100_000n,
 * })
 *
 * const payload = UserOperation.getSignPayload(userOperation, { // [!code focus]
 *   chainId: 1, // [!code focus]
 *   entryPointAddress: '0x1234567890123456789012345678901234567890', // [!code focus]
 *   entryPointVersion: '0.6', // [!code focus]
 * }) // [!code focus]
 *
 * const signature = Secp256k1.sign({ payload, privateKey: '0x...' })
 * ```
 *
 * @param userOperation - The user operation to get the sign payload for.
 * @returns The signing payload for the user operation.
 */
export function getSignPayload(userOperation, options) {
    return hash(userOperation, options);
}
/**
 * Hashes a {@link ox#UserOperation.UserOperation}. This is the "user operation hash".
 *
 * @example
 * ```ts twoslash
 * import { Value } from 'ox'
 * import { UserOperation } from 'ox/erc4337'
 *
 * const userOperation = UserOperation.hash({
 *   callData: '0xdeadbeef',
 *   callGasLimit: 300_000n,
 *   maxFeePerGas: Value.fromGwei('20'),
 *   maxPriorityFeePerGas: Value.fromGwei('2'),
 *   nonce: 69n,
 *   preVerificationGas: 100_000n,
 *   sender: '0x9f1fdab6458c5fc642fa0f4c5af7473c46837357',
 *   verificationGasLimit: 100_000n,
 * }, {
 *   chainId: 1,
 *   entryPointAddress: '0x1234567890123456789012345678901234567890',
 *   entryPointVersion: '0.6',
 * })
 * ```
 *
 * @param userOperation - The user operation to hash.
 * @returns The hash of the user operation.
 */
export function hash(userOperation, options) {
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
/**
 * Converts a {@link ox#UserOperation.UserOperation} to `initCode`.
 *
 * @example
 * ```ts twoslash
 * import { Value } from 'ox'
 * import { UserOperation } from 'ox/erc4337'
 *
 * const initCode = UserOperation.toInitCode({
 *   authorization: {
 *     address: '0x9f1fdab6458c5fc642fa0f4c5af7473c46837357',
 *     chainId: 1,
 *     nonce: 69n,
 *     yParity: 0,
 *     r: 1n,
 *     s: 2n,
 *   },
 *   callData: '0xdeadbeef',
 *   callGasLimit: 300_000n,
 *   factory: '0x7702',
 *   factoryData: '0xdeadbeef',
 *   maxFeePerGas: Value.fromGwei('20'),
 *   maxPriorityFeePerGas: Value.fromGwei('2'),
 *   nonce: 69n,
 *   preVerificationGas: 100_000n,
 *   sender: '0x9f1fdab6458c5fc642fa0f4c5af7473c46837357',
 * })
 * ```
 *
 * @param userOperation - The user operation to convert.
 * @returns The init code.
 */
export function toInitCode(userOperation) {
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
/**
 * Transforms a User Operation into "packed" format.
 *
 * @example
 * ```ts twoslash
 * import { Value } from 'ox'
 * import { UserOperation } from 'ox/erc4337'
 *
 * const packed = UserOperation.toPacked({
 *   callData: '0xdeadbeef',
 *   callGasLimit: 300_000n,
 *   maxFeePerGas: Value.fromGwei('20'),
 *   maxPriorityFeePerGas: Value.fromGwei('2'),
 *   nonce: 69n,
 *   preVerificationGas: 100_000n,
 *   sender: '0x9f1fdab6458c5fc642fa0f4c5af7473c46837357',
 *   signature: '0x...',
 *   verificationGasLimit: 100_000n,
 * })
 * ```
 *
 * @param userOperation - The user operation to transform.
 * @returns The packed user operation.
 */
export function toPacked(userOperation) {
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
/**
 * Transforms a "packed" User Operation into a structured {@link ox#UserOperation.UserOperation}.
 *
 * @example
 * ```ts twoslash
 * import { UserOperation } from 'ox/erc4337'
 *
 * const packed: UserOperation.Packed = {
 *   accountGasLimits: '0x...',
 *   callData: '0xdeadbeef',
 *   initCode: '0x...',
 *   gasFees: '0x...',
 *   nonce: 69n,
 *   paymasterAndData: '0x',
 *   preVerificationGas: 100_000n,
 *   sender: '0x9f1fdab6458c5fc642fa0f4c5af7473c46837357',
 *   signature: '0x...',
 * }
 *
 * const userOperation = UserOperation.fromPacked(packed)
 * ```
 *
 * @param packed - The packed user operation to transform.
 * @returns The structured user operation.
 */
export function fromPacked(packed) {
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
/**
 * Converts a {@link ox#UserOperation.UserOperation} to a {@link ox#UserOperation.Rpc}.
 *
 * @example
 * ```ts twoslash
 * import { Value } from 'ox'
 * import { UserOperation } from 'ox/erc4337'
 *
 * const userOperation = UserOperation.toRpc({
 *   callData: '0xdeadbeef',
 *   callGasLimit: 300_000n,
 *   maxFeePerGas: Value.fromGwei('20'),
 *   maxPriorityFeePerGas: Value.fromGwei('2'),
 *   nonce: 69n,
 *   preVerificationGas: 100_000n,
 *   sender: '0x9f1fdab6458c5fc642fa0f4c5af7473c46837357',
 *   verificationGasLimit: 100_000n,
 * })
 * ```
 *
 * @param userOperation - The user operation to convert.
 * @returns An RPC-formatted user operation.
 */
export function toRpc(userOperation) {
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
/**
 * Converts a signed {@link ox#UserOperation.UserOperation} to a {@link ox#TypedData.Definition}.
 *
 * @example
 * ```ts twoslash
 * import { Value } from 'ox'
 * import { UserOperation } from 'ox/erc4337'
 *
 * const typedData = UserOperation.toTypedData({
 *   authorization: {
 *     chainId: 1,
 *     address: '0x9f1fdab6458c5fc642fa0f4c5af7473c46837357',
 *     nonce: 69n,
 *     yParity: 0,
 *     r: 1n,
 *     s: 2n,
 *   },
 *   callData: '0xdeadbeef',
 *   callGasLimit: 300_000n,
 *   maxFeePerGas: Value.fromGwei('20'),
 *   maxPriorityFeePerGas: Value.fromGwei('2'),
 *   nonce: 69n,
 *   preVerificationGas: 100_000n,
 *   sender: '0x9f1fdab6458c5fc642fa0f4c5af7473c46837357',
 *   signature: '0x...',
 *   verificationGasLimit: 100_000n,
 * }, {
 *   chainId: 1,
 *   entryPointAddress: '0x1234567890123456789012345678901234567890',
 * })
 * ```
 *
 * @param userOperation - The user operation to convert.
 * @returns A Typed Data definition.
 */
export function toTypedData(userOperation, options) {
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
})(toTypedData || (toTypedData = {}));
//# sourceMappingURL=UserOperation.js.map