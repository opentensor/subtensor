import * as AbiParameters from '../core/AbiParameters.js';
import type * as Address from '../core/Address.js';
import type * as Authorization from '../core/Authorization.js';
import type * as Errors from '../core/Errors.js';
import * as Hash from '../core/Hash.js';
import * as Hex from '../core/Hex.js';
import type { Assign, Compute, OneOf } from '../core/internal/types.js';
import * as Signature from '../core/Signature.js';
import * as TypedData from '../core/TypedData.js';
import type * as EntryPoint from './EntryPoint.js';
/** User Operation. */
export type UserOperation<entryPointVersion extends EntryPoint.Version = EntryPoint.Version, signed extends boolean = boolean, bigintType = bigint, numberType = number> = OneOf<(entryPointVersion extends '0.6' ? V06<signed, bigintType> : never) | (entryPointVersion extends '0.7' ? V07<signed, bigintType> : never) | (entryPointVersion extends '0.8' ? V08<signed, bigintType, numberType> : never)>;
/**
 * Packed User Operation.
 *
 * @see https://eips.ethereum.org/EIPS/eip-4337#entrypoint-definition
 */
export type Packed = {
    /** Concatenation of `verificationGasLimit` (16 bytes) and `callGasLimit` (16 bytes) */
    accountGasLimits: Hex.Hex;
    /** The data to pass to the `sender` during the main execution call. */
    callData: Hex.Hex;
    /** Concatenation of `factory` and `factoryData`. */
    initCode: Hex.Hex;
    /** Concatenation of `maxPriorityFee` (16 bytes) and `maxFeePerGas` (16 bytes) */
    gasFees: Hex.Hex;
    /** Anti-replay parameter. */
    nonce: bigint;
    /** Concatenation of paymaster fields (or empty). */
    paymasterAndData: Hex.Hex;
    /** Extra gas to pay the Bundler. */
    preVerificationGas: bigint;
    /** The account making the operation. */
    sender: Address.Address;
    /** Data passed into the account to verify authorization. */
    signature: Hex.Hex;
};
/** RPC User Operation type. */
export type Rpc<entryPointVersion extends EntryPoint.Version = EntryPoint.Version, signed extends boolean = true> = OneOf<(entryPointVersion extends '0.6' ? V06<signed, Hex.Hex> : never) | (entryPointVersion extends '0.7' ? V07<signed, Hex.Hex> : never) | (entryPointVersion extends '0.8' ? V08<signed, Hex.Hex, Hex.Hex> : never)>;
/** Transaction Info. */
export type TransactionInfo<entryPointVersion extends EntryPoint.Version = EntryPoint.Version, bigintType = bigint> = {
    blockHash: Hex.Hex;
    blockNumber: bigintType;
    entryPoint: Address.Address;
    transactionHash: Hex.Hex;
    userOperation: UserOperation<entryPointVersion, true, bigintType>;
};
/** RPC Transaction Info. */
export type RpcTransactionInfo<entryPointVersion extends EntryPoint.Version = EntryPoint.Version> = TransactionInfo<entryPointVersion, Hex.Hex>;
/** Type for User Operation on EntryPoint 0.6 */
export type V06<signed extends boolean = boolean, bigintType = bigint> = {
    /** The data to pass to the `sender` during the main execution call. */
    callData: Hex.Hex;
    /** The amount of gas to allocate the main execution call */
    callGasLimit: bigintType;
    /** Account init code. Only for new accounts. */
    initCode?: Hex.Hex | undefined;
    /** Maximum fee per gas. */
    maxFeePerGas: bigintType;
    /** Maximum priority fee per gas. */
    maxPriorityFeePerGas: bigintType;
    /** Anti-replay parameter. */
    nonce: bigintType;
    /** Paymaster address with calldata. */
    paymasterAndData?: Hex.Hex | undefined;
    /** Extra gas to pay the Bundler. */
    preVerificationGas: bigintType;
    /** The account making the operation. */
    sender: Address.Address;
    /** Data passed into the account to verify authorization. */
    signature?: Hex.Hex | undefined;
    /** The amount of gas to allocate for the verification step. */
    verificationGasLimit: bigintType;
} & (signed extends true ? {
    signature: Hex.Hex;
} : {});
/** RPC User Operation on EntryPoint 0.6 */
export type RpcV06<signed extends boolean = true> = V06<signed, Hex.Hex>;
/** Type for User Operation on EntryPoint 0.7 */
export type V07<signed extends boolean = boolean, bigintType = bigint> = {
    /** The data to pass to the `sender` during the main execution call. */
    callData: Hex.Hex;
    /** The amount of gas to allocate the main execution call */
    callGasLimit: bigintType;
    /** Account factory. Only for new accounts. */
    factory?: Address.Address | undefined;
    /** Data for account factory. */
    factoryData?: Hex.Hex | undefined;
    /** Maximum fee per gas. */
    maxFeePerGas: bigintType;
    /** Maximum priority fee per gas. */
    maxPriorityFeePerGas: bigintType;
    /** Anti-replay parameter. */
    nonce: bigintType;
    /** Address of paymaster contract. */
    paymaster?: Address.Address | undefined;
    /** Data for paymaster. */
    paymasterData?: Hex.Hex | undefined;
    /** The amount of gas to allocate for the paymaster post-operation code. */
    paymasterPostOpGasLimit?: bigintType | undefined;
    /** The amount of gas to allocate for the paymaster validation code. */
    paymasterVerificationGasLimit?: bigintType | undefined;
    /** Extra gas to pay the Bundler. */
    preVerificationGas: bigintType;
    /** The account making the operation. */
    sender: Address.Address;
    /** Data passed into the account to verify authorization. */
    signature?: Hex.Hex | undefined;
    /** The amount of gas to allocate for the verification step. */
    verificationGasLimit: bigintType;
} & (signed extends true ? {
    signature: Hex.Hex;
} : {});
/** RPC User Operation on EntryPoint 0.7 */
export type RpcV07<signed extends boolean = true> = V07<signed, Hex.Hex>;
/** Type for User Operation on EntryPoint 0.8 */
export type V08<signed extends boolean = boolean, bigintType = bigint, numberType = number> = {
    /** Authorization data. */
    authorization?: Authorization.Signed<bigintType, numberType> | undefined;
    /** The data to pass to the `sender` during the main execution call. */
    callData: Hex.Hex;
    /** The amount of gas to allocate the main execution call */
    callGasLimit: bigintType;
    /** Account factory. Only for new accounts. */
    factory?: Address.Address | undefined;
    /** Data for account factory. */
    factoryData?: Hex.Hex | undefined;
    /** Maximum fee per gas. */
    maxFeePerGas: bigintType;
    /** Maximum priority fee per gas. */
    maxPriorityFeePerGas: bigintType;
    /** Anti-replay parameter. */
    nonce: bigintType;
    /** Address of paymaster contract. */
    paymaster?: Address.Address | undefined;
    /** Data for paymaster. */
    paymasterData?: Hex.Hex | undefined;
    /** The amount of gas to allocate for the paymaster post-operation code. */
    paymasterPostOpGasLimit?: bigintType | undefined;
    /** The amount of gas to allocate for the paymaster validation code. */
    paymasterVerificationGasLimit?: bigintType | undefined;
    /** Extra gas to pay the Bundler. */
    preVerificationGas: bigintType;
    /** The account making the operation. */
    sender: Address.Address;
    /** Data passed into the account to verify authorization. */
    signature?: Hex.Hex | undefined;
    /** The amount of gas to allocate for the verification step. */
    verificationGasLimit: bigintType;
} & (signed extends true ? {
    signature: Hex.Hex;
} : {});
/** RPC User Operation on EntryPoint 0.8 */
export type RpcV08<signed extends boolean = true> = V08<signed, Hex.Hex, Hex.Hex>;
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
export declare function from<const userOperation extends UserOperation | Packed, const signature extends Hex.Hex | undefined = undefined>(userOperation: userOperation | UserOperation | Packed, options?: from.Options<signature>): from.ReturnType<userOperation, signature>;
export declare namespace from {
    type Options<signature extends Signature.Signature | Hex.Hex | undefined = undefined> = {
        signature?: signature | Signature.Signature | Hex.Hex | undefined;
    };
    type ReturnType<userOperation extends UserOperation | Packed = UserOperation | Packed, signature extends Signature.Signature | Hex.Hex | undefined = undefined> = Compute<Assign<userOperation, signature extends Signature.Signature | Hex.Hex ? Readonly<{
        signature: Hex.Hex;
    }> : {}>>;
    type ErrorType = Errors.GlobalErrorType;
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
export declare function fromRpc(rpc: Rpc): UserOperation;
export declare namespace fromRpc {
    type ErrorType = Errors.GlobalErrorType;
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
export declare function getSignPayload<entrypointVersion extends EntryPoint.Version = EntryPoint.Version>(userOperation: UserOperation<entrypointVersion>, options: getSignPayload.Options<entrypointVersion>): Hex.Hex;
export declare namespace getSignPayload {
    type Options<entrypointVersion extends EntryPoint.Version = EntryPoint.Version> = hash.Options<entrypointVersion>;
    type ErrorType = hash.ErrorType | Errors.GlobalErrorType;
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
export declare function hash<entrypointVersion extends EntryPoint.Version = EntryPoint.Version>(userOperation: UserOperation<entrypointVersion>, options: hash.Options<entrypointVersion>): Hex.Hex;
export declare namespace hash {
    type Options<entrypointVersion extends EntryPoint.Version = EntryPoint.Version> = {
        chainId: number;
        entryPointAddress: Address.Address;
        entryPointVersion: entrypointVersion | EntryPoint.Version;
    };
    type ErrorType = AbiParameters.encode.ErrorType | Hash.keccak256.ErrorType | Hex.concat.ErrorType | Hex.fromNumber.ErrorType | Hex.padLeft.ErrorType | Errors.GlobalErrorType;
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
export declare function toInitCode(userOperation: Partial<UserOperation>): Hex.Hex;
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
export declare function toPacked(userOperation: UserOperation<'0.7' | '0.8', true>): Packed;
export declare namespace toPacked {
    type ErrorType = Errors.GlobalErrorType;
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
export declare function fromPacked(packed: Packed): UserOperation<'0.7' | '0.8', true>;
export declare namespace fromPacked {
    type ErrorType = Hex.slice.ErrorType | Errors.GlobalErrorType;
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
export declare function toRpc(userOperation: UserOperation): Rpc;
export declare namespace toRpc {
    type ErrorType = Hex.fromNumber.ErrorType | Errors.GlobalErrorType;
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
export declare function toTypedData(userOperation: UserOperation<'0.8', true>, options: toTypedData.Options): TypedData.Definition<typeof toTypedData.types, 'PackedUserOperation'>;
export declare namespace toTypedData {
    type Options = {
        chainId: number;
        entryPointAddress: Address.Address;
    };
    type ErrorType = Errors.GlobalErrorType;
    const types: {
        readonly PackedUserOperation: readonly [{
            readonly type: "address";
            readonly name: "sender";
        }, {
            readonly type: "uint256";
            readonly name: "nonce";
        }, {
            readonly type: "bytes";
            readonly name: "initCode";
        }, {
            readonly type: "bytes";
            readonly name: "callData";
        }, {
            readonly type: "bytes32";
            readonly name: "accountGasLimits";
        }, {
            readonly type: "uint256";
            readonly name: "preVerificationGas";
        }, {
            readonly type: "bytes32";
            readonly name: "gasFees";
        }, {
            readonly type: "bytes";
            readonly name: "paymasterAndData";
        }];
    };
}
//# sourceMappingURL=UserOperation.d.ts.map