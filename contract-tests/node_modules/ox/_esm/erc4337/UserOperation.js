import * as AbiParameters from '../core/AbiParameters.js';
import * as Hash from '../core/Hash.js';
import * as Hex from '../core/Hex.js';
import * as Signature from '../core/Signature.js';
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
 * @param userOperation - The user operation to instantiate.
 * @returns User Operation.
 */
export function from(userOperation, options = {}) {
    const signature = (() => {
        if (!options.signature)
            return undefined;
        if (typeof options.signature === 'string')
            return options.signature;
        return Signature.toHex(options.signature);
    })();
    return { ...userOperation, signature };
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
//# sourceMappingURL=UserOperation.js.map