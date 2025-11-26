"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getL2TokenAddress = getL2TokenAddress;
const readContract_js_1 = require("../../actions/public/readContract.js");
const index_js_1 = require("../../utils/index.js");
const abis_js_1 = require("../constants/abis.js");
const address_js_1 = require("../constants/address.js");
const getBaseTokenL1Address_js_1 = require("./getBaseTokenL1Address.js");
const getDefaultBridgeAddresses_js_1 = require("./getDefaultBridgeAddresses.js");
async function getL2TokenAddress(client, parameters) {
    let { token, bridgeAddress } = parameters;
    if ((0, index_js_1.isAddressEqual)(token, address_js_1.legacyEthAddress))
        token = address_js_1.ethAddressInContracts;
    const baseToken = await (0, getBaseTokenL1Address_js_1.getBaseTokenL1Address)(client);
    if ((0, index_js_1.isAddressEqual)(token, baseToken))
        return address_js_1.l2BaseTokenAddress;
    bridgeAddress ??= (await (0, getDefaultBridgeAddresses_js_1.getDefaultBridgeAddresses)(client)).sharedL2;
    return await (0, readContract_js_1.readContract)(client, {
        address: bridgeAddress,
        abi: abis_js_1.l2SharedBridgeAbi,
        functionName: 'l2TokenAddress',
        args: [token],
    });
}
//# sourceMappingURL=getL2TokenAddress.js.map