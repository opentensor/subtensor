"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getEnsAddress = getEnsAddress;
const abis_js_1 = require("../../constants/abis.js");
const decodeFunctionResult_js_1 = require("../../utils/abi/decodeFunctionResult.js");
const encodeFunctionData_js_1 = require("../../utils/abi/encodeFunctionData.js");
const getChainContractAddress_js_1 = require("../../utils/chain/getChainContractAddress.js");
const trim_js_1 = require("../../utils/data/trim.js");
const toHex_js_1 = require("../../utils/encoding/toHex.js");
const errors_js_1 = require("../../utils/ens/errors.js");
const localBatchGatewayRequest_js_1 = require("../../utils/ens/localBatchGatewayRequest.js");
const namehash_js_1 = require("../../utils/ens/namehash.js");
const packetToBytes_js_1 = require("../../utils/ens/packetToBytes.js");
const getAction_js_1 = require("../../utils/getAction.js");
const readContract_js_1 = require("../public/readContract.js");
async function getEnsAddress(client, parameters) {
    const { blockNumber, blockTag, coinType, name, gatewayUrls, strict } = parameters;
    const { chain } = client;
    const universalResolverAddress = (() => {
        if (parameters.universalResolverAddress)
            return parameters.universalResolverAddress;
        if (!chain)
            throw new Error('client chain not configured. universalResolverAddress is required.');
        return (0, getChainContractAddress_js_1.getChainContractAddress)({
            blockNumber,
            chain,
            contract: 'ensUniversalResolver',
        });
    })();
    const tlds = chain?.ensTlds;
    if (tlds && !tlds.some((tld) => name.endsWith(tld)))
        return null;
    const args = (() => {
        if (coinType != null)
            return [(0, namehash_js_1.namehash)(name), BigInt(coinType)];
        return [(0, namehash_js_1.namehash)(name)];
    })();
    try {
        const functionData = (0, encodeFunctionData_js_1.encodeFunctionData)({
            abi: abis_js_1.addressResolverAbi,
            functionName: 'addr',
            args,
        });
        const readContractParameters = {
            address: universalResolverAddress,
            abi: abis_js_1.universalResolverResolveAbi,
            functionName: 'resolveWithGateways',
            args: [
                (0, toHex_js_1.toHex)((0, packetToBytes_js_1.packetToBytes)(name)),
                functionData,
                gatewayUrls ?? [localBatchGatewayRequest_js_1.localBatchGatewayUrl],
            ],
            blockNumber,
            blockTag,
        };
        const readContractAction = (0, getAction_js_1.getAction)(client, readContract_js_1.readContract, 'readContract');
        const res = await readContractAction(readContractParameters);
        if (res[0] === '0x')
            return null;
        const address = (0, decodeFunctionResult_js_1.decodeFunctionResult)({
            abi: abis_js_1.addressResolverAbi,
            args,
            functionName: 'addr',
            data: res[0],
        });
        if (address === '0x')
            return null;
        if ((0, trim_js_1.trim)(address) === '0x00')
            return null;
        return address;
    }
    catch (err) {
        if (strict)
            throw err;
        if ((0, errors_js_1.isNullUniversalResolverError)(err))
            return null;
        throw err;
    }
}
//# sourceMappingURL=getEnsAddress.js.map