"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getEnsName = getEnsName;
const abis_js_1 = require("../../constants/abis.js");
const getChainContractAddress_js_1 = require("../../utils/chain/getChainContractAddress.js");
const errors_js_1 = require("../../utils/ens/errors.js");
const localBatchGatewayRequest_js_1 = require("../../utils/ens/localBatchGatewayRequest.js");
const getAction_js_1 = require("../../utils/getAction.js");
const readContract_js_1 = require("../public/readContract.js");
async function getEnsName(client, parameters) {
    const { address, blockNumber, blockTag, coinType = 60n, gatewayUrls, strict, } = parameters;
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
    try {
        const readContractParameters = {
            address: universalResolverAddress,
            abi: abis_js_1.universalResolverReverseAbi,
            args: [address, coinType, gatewayUrls ?? [localBatchGatewayRequest_js_1.localBatchGatewayUrl]],
            functionName: 'reverseWithGateways',
            blockNumber,
            blockTag,
        };
        const readContractAction = (0, getAction_js_1.getAction)(client, readContract_js_1.readContract, 'readContract');
        const [name] = await readContractAction(readContractParameters);
        return name || null;
    }
    catch (err) {
        if (strict)
            throw err;
        if ((0, errors_js_1.isNullUniversalResolverError)(err))
            return null;
        throw err;
    }
}
//# sourceMappingURL=getEnsName.js.map