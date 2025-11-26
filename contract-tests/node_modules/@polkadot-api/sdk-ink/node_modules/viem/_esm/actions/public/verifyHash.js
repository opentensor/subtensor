import { SignatureErc6492 } from 'ox/erc6492';
import { SignatureErc8010 } from 'ox/erc8010';
import { erc1271Abi, erc6492SignatureValidatorAbi, multicall3Abi, } from '../../constants/abis.js';
import { erc6492SignatureValidatorByteCode, multicall3Bytecode, } from '../../constants/contracts.js';
import { CallExecutionError, ContractFunctionExecutionError, } from '../../errors/contract.js';
import { encodeDeployData, } from '../../utils/abi/encodeDeployData.js';
import { encodeFunctionData, } from '../../utils/abi/encodeFunctionData.js';
import { getAddress, } from '../../utils/address/getAddress.js';
import { isAddressEqual, } from '../../utils/address/isAddressEqual.js';
import { verifyAuthorization } from '../../utils/authorization/verifyAuthorization.js';
import { concatHex } from '../../utils/data/concat.js';
import { isHex } from '../../utils/data/isHex.js';
import { hexToBool } from '../../utils/encoding/fromHex.js';
import { bytesToHex, numberToHex, } from '../../utils/encoding/toHex.js';
import { getAction } from '../../utils/getAction.js';
import { recoverAddress, } from '../../utils/signature/recoverAddress.js';
import { serializeSignature, } from '../../utils/signature/serializeSignature.js';
import { call } from './call.js';
import { getCode } from './getCode.js';
import { readContract } from './readContract.js';
/**
 * Verifies a message hash onchain using ERC-6492.
 *
 * @param client - Client to use.
 * @param parameters - {@link VerifyHashParameters}
 * @returns Whether or not the signature is valid. {@link VerifyHashReturnType}
 */
export async function verifyHash(client, parameters) {
    const { address, hash, erc6492VerifierAddress: verifierAddress = parameters.universalSignatureVerifierAddress ??
        client.chain?.contracts?.erc6492Verifier?.address, multicallAddress = parameters.multicallAddress ??
        client.chain?.contracts?.multicall3?.address, } = parameters;
    const signature = (() => {
        const signature = parameters.signature;
        if (isHex(signature))
            return signature;
        if (typeof signature === 'object' && 'r' in signature && 's' in signature)
            return serializeSignature(signature);
        return bytesToHex(signature);
    })();
    try {
        if (SignatureErc8010.validate(signature))
            return await verifyErc8010(client, {
                ...parameters,
                multicallAddress,
                signature,
            });
        return await verifyErc6492(client, {
            ...parameters,
            verifierAddress,
            signature,
        });
    }
    catch (error) {
        // Fallback attempt to verify the signature via ECDSA recovery.
        try {
            const verified = isAddressEqual(getAddress(address), await recoverAddress({ hash, signature }));
            if (verified)
                return true;
        }
        catch { }
        if (error instanceof VerificationError) {
            // if the execution fails, the signature was not valid and an internal method inside of the validator reverted
            // this can happen for many reasons, for example if signer can not be recovered from the signature
            // or if the signature has no valid format
            return false;
        }
        throw error;
    }
}
/** @internal */
export async function verifyErc8010(client, parameters) {
    const { address, blockNumber, blockTag, hash, multicallAddress } = parameters;
    const { authorization: authorization_ox, data: initData, signature, to, } = SignatureErc8010.unwrap(parameters.signature);
    // Check if already delegated
    const code = await getCode(client, {
        address,
        blockNumber,
        blockTag,
    });
    // If already delegated, perform standard ERC-1271 verification.
    if (code === concatHex(['0xef0100', authorization_ox.address]))
        return await verifyErc1271(client, {
            address,
            blockNumber,
            blockTag,
            hash,
            signature,
        });
    const authorization = {
        address: authorization_ox.address,
        chainId: Number(authorization_ox.chainId),
        nonce: Number(authorization_ox.nonce),
        r: numberToHex(authorization_ox.r, { size: 32 }),
        s: numberToHex(authorization_ox.s, { size: 32 }),
        yParity: authorization_ox.yParity,
    };
    const valid = await verifyAuthorization({
        address,
        authorization,
    });
    if (!valid)
        throw new VerificationError();
    // Deployless verification.
    const results = await getAction(client, readContract, 'readContract')({
        ...(multicallAddress
            ? { address: multicallAddress }
            : { code: multicall3Bytecode }),
        authorizationList: [authorization],
        abi: multicall3Abi,
        blockNumber,
        blockTag: 'pending',
        functionName: 'aggregate3',
        args: [
            [
                ...(initData
                    ? [
                        {
                            allowFailure: true,
                            target: to ?? address,
                            callData: initData,
                        },
                    ]
                    : []),
                {
                    allowFailure: true,
                    target: address,
                    callData: encodeFunctionData({
                        abi: erc1271Abi,
                        functionName: 'isValidSignature',
                        args: [hash, signature],
                    }),
                },
            ],
        ],
    });
    const data = results[results.length - 1]?.returnData;
    if (data?.startsWith('0x1626ba7e'))
        return true;
    throw new VerificationError();
}
/** @internal */
// biome-ignore lint/correctness/noUnusedVariables: _
async function verifyErc6492(client, parameters) {
    const { address, factory, factoryData, hash, signature, verifierAddress, ...rest } = parameters;
    const wrappedSignature = await (async () => {
        // If no `factory` or `factoryData` is provided, it is assumed that the
        // address is not a Smart Account, or the Smart Account is already deployed.
        if (!factory && !factoryData)
            return signature;
        // If the signature is already wrapped, return the signature.
        if (SignatureErc6492.validate(signature))
            return signature;
        // If the Smart Account is not deployed, wrap the signature with a 6492 wrapper
        // to perform counterfactual validation.
        return SignatureErc6492.wrap({
            data: factoryData,
            signature,
            to: factory,
        });
    })();
    const args = verifierAddress
        ? {
            to: verifierAddress,
            data: encodeFunctionData({
                abi: erc6492SignatureValidatorAbi,
                functionName: 'isValidSig',
                args: [address, hash, wrappedSignature],
            }),
            ...rest,
        }
        : {
            data: encodeDeployData({
                abi: erc6492SignatureValidatorAbi,
                args: [address, hash, wrappedSignature],
                bytecode: erc6492SignatureValidatorByteCode,
            }),
            ...rest,
        };
    const { data } = await getAction(client, call, 'call')(args).catch((error) => {
        if (error instanceof CallExecutionError)
            throw new VerificationError();
        throw error;
    });
    if (hexToBool(data ?? '0x0'))
        return true;
    throw new VerificationError();
}
/** @internal */
export async function verifyErc1271(client, parameters) {
    const { address, blockNumber, blockTag, hash, signature } = parameters;
    const result = await getAction(client, readContract, 'readContract')({
        address,
        abi: erc1271Abi,
        args: [hash, signature],
        blockNumber,
        blockTag,
        functionName: 'isValidSignature',
    }).catch((error) => {
        if (error instanceof ContractFunctionExecutionError)
            throw new VerificationError();
        throw error;
    });
    if (result.startsWith('0x1626ba7e'))
        return true;
    throw new VerificationError();
}
class VerificationError extends Error {
}
//# sourceMappingURL=verifyHash.js.map