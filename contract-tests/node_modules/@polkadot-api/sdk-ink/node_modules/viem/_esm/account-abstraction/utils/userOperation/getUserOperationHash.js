import { encodeAbiParameters } from '../../../utils/abi/encodeAbiParameters.js';
import { keccak256 } from '../../../utils/hash/keccak256.js';
import { hashTypedData } from '../../../utils/signature/hashTypedData.js';
import { getInitCode } from './getInitCode.js';
import { getUserOperationTypedData } from './getUserOperationTypedData.js';
import { toPackedUserOperation } from './toPackedUserOperation.js';
export function getUserOperationHash(parameters) {
    const { chainId, entryPointAddress, entryPointVersion } = parameters;
    const userOperation = parameters.userOperation;
    const { authorization, callData = '0x', callGasLimit, maxFeePerGas, maxPriorityFeePerGas, nonce, paymasterAndData = '0x', preVerificationGas, sender, verificationGasLimit, } = userOperation;
    if (entryPointVersion === '0.8')
        return hashTypedData(getUserOperationTypedData({
            chainId,
            entryPointAddress,
            userOperation,
        }));
    const packedUserOp = (() => {
        if (entryPointVersion === '0.6') {
            const factory = userOperation.initCode?.slice(0, 42);
            const factoryData = userOperation.initCode?.slice(42);
            const initCode = getInitCode({
                authorization,
                factory,
                factoryData,
            });
            return encodeAbiParameters([
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
                keccak256(initCode),
                keccak256(callData),
                callGasLimit,
                verificationGasLimit,
                preVerificationGas,
                maxFeePerGas,
                maxPriorityFeePerGas,
                keccak256(paymasterAndData),
            ]);
        }
        if (entryPointVersion === '0.7') {
            const packedUserOp = toPackedUserOperation(userOperation);
            return encodeAbiParameters([
                { type: 'address' },
                { type: 'uint256' },
                { type: 'bytes32' },
                { type: 'bytes32' },
                { type: 'bytes32' },
                { type: 'uint256' },
                { type: 'bytes32' },
                { type: 'bytes32' },
            ], [
                packedUserOp.sender,
                packedUserOp.nonce,
                keccak256(packedUserOp.initCode),
                keccak256(packedUserOp.callData),
                packedUserOp.accountGasLimits,
                packedUserOp.preVerificationGas,
                packedUserOp.gasFees,
                keccak256(packedUserOp.paymasterAndData),
            ]);
        }
        throw new Error(`entryPointVersion "${entryPointVersion}" not supported.`);
    })();
    return keccak256(encodeAbiParameters([{ type: 'bytes32' }, { type: 'address' }, { type: 'uint256' }], [keccak256(packedUserOp), entryPointAddress, BigInt(chainId)]));
}
//# sourceMappingURL=getUserOperationHash.js.map