"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.verifyHash = verifyHash;
exports.verifyErc8010 = verifyErc8010;
exports.verifyErc1271 = verifyErc1271;
const erc6492_1 = require("ox/erc6492");
const erc8010_1 = require("ox/erc8010");
const abis_js_1 = require("../../constants/abis.js");
const contracts_js_1 = require("../../constants/contracts.js");
const contract_js_1 = require("../../errors/contract.js");
const encodeDeployData_js_1 = require("../../utils/abi/encodeDeployData.js");
const encodeFunctionData_js_1 = require("../../utils/abi/encodeFunctionData.js");
const getAddress_js_1 = require("../../utils/address/getAddress.js");
const isAddressEqual_js_1 = require("../../utils/address/isAddressEqual.js");
const verifyAuthorization_js_1 = require("../../utils/authorization/verifyAuthorization.js");
const concat_js_1 = require("../../utils/data/concat.js");
const isHex_js_1 = require("../../utils/data/isHex.js");
const fromHex_js_1 = require("../../utils/encoding/fromHex.js");
const toHex_js_1 = require("../../utils/encoding/toHex.js");
const getAction_js_1 = require("../../utils/getAction.js");
const recoverAddress_js_1 = require("../../utils/signature/recoverAddress.js");
const serializeSignature_js_1 = require("../../utils/signature/serializeSignature.js");
const call_js_1 = require("./call.js");
const getCode_js_1 = require("./getCode.js");
const readContract_js_1 = require("./readContract.js");
async function verifyHash(client, parameters) {
    const { address, hash, erc6492VerifierAddress: verifierAddress = parameters.universalSignatureVerifierAddress ??
        client.chain?.contracts?.erc6492Verifier?.address, multicallAddress = parameters.multicallAddress ??
        client.chain?.contracts?.multicall3?.address, } = parameters;
    const signature = (() => {
        const signature = parameters.signature;
        if ((0, isHex_js_1.isHex)(signature))
            return signature;
        if (typeof signature === 'object' && 'r' in signature && 's' in signature)
            return (0, serializeSignature_js_1.serializeSignature)(signature);
        return (0, toHex_js_1.bytesToHex)(signature);
    })();
    try {
        if (erc8010_1.SignatureErc8010.validate(signature))
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
        try {
            const verified = (0, isAddressEqual_js_1.isAddressEqual)((0, getAddress_js_1.getAddress)(address), await (0, recoverAddress_js_1.recoverAddress)({ hash, signature }));
            if (verified)
                return true;
        }
        catch { }
        if (error instanceof VerificationError) {
            return false;
        }
        throw error;
    }
}
async function verifyErc8010(client, parameters) {
    const { address, blockNumber, blockTag, hash, multicallAddress } = parameters;
    const { authorization: authorization_ox, data: initData, signature, to, } = erc8010_1.SignatureErc8010.unwrap(parameters.signature);
    const code = await (0, getCode_js_1.getCode)(client, {
        address,
        blockNumber,
        blockTag,
    });
    if (code === (0, concat_js_1.concatHex)(['0xef0100', authorization_ox.address]))
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
        r: (0, toHex_js_1.numberToHex)(authorization_ox.r, { size: 32 }),
        s: (0, toHex_js_1.numberToHex)(authorization_ox.s, { size: 32 }),
        yParity: authorization_ox.yParity,
    };
    const valid = await (0, verifyAuthorization_js_1.verifyAuthorization)({
        address,
        authorization,
    });
    if (!valid)
        throw new VerificationError();
    const results = await (0, getAction_js_1.getAction)(client, readContract_js_1.readContract, 'readContract')({
        ...(multicallAddress
            ? { address: multicallAddress }
            : { code: contracts_js_1.multicall3Bytecode }),
        authorizationList: [authorization],
        abi: abis_js_1.multicall3Abi,
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
                    callData: (0, encodeFunctionData_js_1.encodeFunctionData)({
                        abi: abis_js_1.erc1271Abi,
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
async function verifyErc6492(client, parameters) {
    const { address, factory, factoryData, hash, signature, verifierAddress, ...rest } = parameters;
    const wrappedSignature = await (async () => {
        if (!factory && !factoryData)
            return signature;
        if (erc6492_1.SignatureErc6492.validate(signature))
            return signature;
        return erc6492_1.SignatureErc6492.wrap({
            data: factoryData,
            signature,
            to: factory,
        });
    })();
    const args = verifierAddress
        ? {
            to: verifierAddress,
            data: (0, encodeFunctionData_js_1.encodeFunctionData)({
                abi: abis_js_1.erc6492SignatureValidatorAbi,
                functionName: 'isValidSig',
                args: [address, hash, wrappedSignature],
            }),
            ...rest,
        }
        : {
            data: (0, encodeDeployData_js_1.encodeDeployData)({
                abi: abis_js_1.erc6492SignatureValidatorAbi,
                args: [address, hash, wrappedSignature],
                bytecode: contracts_js_1.erc6492SignatureValidatorByteCode,
            }),
            ...rest,
        };
    const { data } = await (0, getAction_js_1.getAction)(client, call_js_1.call, 'call')(args).catch((error) => {
        if (error instanceof contract_js_1.CallExecutionError)
            throw new VerificationError();
        throw error;
    });
    if ((0, fromHex_js_1.hexToBool)(data ?? '0x0'))
        return true;
    throw new VerificationError();
}
async function verifyErc1271(client, parameters) {
    const { address, blockNumber, blockTag, hash, signature } = parameters;
    const result = await (0, getAction_js_1.getAction)(client, readContract_js_1.readContract, 'readContract')({
        address,
        abi: abis_js_1.erc1271Abi,
        args: [hash, signature],
        blockNumber,
        blockTag,
        functionName: 'isValidSignature',
    }).catch((error) => {
        if (error instanceof contract_js_1.ContractFunctionExecutionError)
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