"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getL1TokenAddress = getL1TokenAddress;
const readContract_js_1 = require("../../actions/public/readContract.js");
const index_js_1 = require("../../utils/index.js");
const abis_js_1 = require("../constants/abis.js");
const address_js_1 = require("../constants/address.js");
const getDefaultBridgeAddresses_js_1 = require("./getDefaultBridgeAddresses.js");
async function getL1TokenAddress(client, parameters) {
    const { token } = parameters;
    if ((0, index_js_1.isAddressEqual)(token, address_js_1.legacyEthAddress))
        return address_js_1.legacyEthAddress;
    const bridgeAddress = (await (0, getDefaultBridgeAddresses_js_1.getDefaultBridgeAddresses)(client)).sharedL2;
    return await (0, readContract_js_1.readContract)(client, {
        address: bridgeAddress,
        abi: abis_js_1.l2SharedBridgeAbi,
        functionName: 'l1TokenAddress',
        args: [token],
    });
}
//# sourceMappingURL=getL1TokenAddress.js.map