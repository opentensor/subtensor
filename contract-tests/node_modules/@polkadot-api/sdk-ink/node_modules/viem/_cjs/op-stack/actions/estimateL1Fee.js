"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.estimateL1Fee = estimateL1Fee;
const readContract_js_1 = require("../../actions/public/readContract.js");
const getChainContractAddress_js_1 = require("../../utils/chain/getChainContractAddress.js");
const serializeTransaction_js_1 = require("../../utils/transaction/serializeTransaction.js");
const parseGwei_js_1 = require("../../utils/unit/parseGwei.js");
const abis_js_1 = require("../abis.js");
const contracts_js_1 = require("../contracts.js");
async function estimateL1Fee(client, args) {
    const { chain = client.chain, gasPriceOracleAddress: gasPriceOracleAddress_, } = args;
    const gasPriceOracleAddress = (() => {
        if (gasPriceOracleAddress_)
            return gasPriceOracleAddress_;
        if (chain)
            return (0, getChainContractAddress_js_1.getChainContractAddress)({
                chain,
                contract: 'gasPriceOracle',
            });
        return contracts_js_1.contracts.gasPriceOracle.address;
    })();
    const transaction = (0, serializeTransaction_js_1.serializeTransaction)({
        ...args,
        chainId: chain?.id ?? 1,
        type: 'eip1559',
        gas: args.data ? 300000n : 21000n,
        maxFeePerGas: (0, parseGwei_js_1.parseGwei)('5'),
        maxPriorityFeePerGas: (0, parseGwei_js_1.parseGwei)('1'),
        nonce: 1,
    });
    return (0, readContract_js_1.readContract)(client, {
        abi: abis_js_1.gasPriceOracleAbi,
        address: gasPriceOracleAddress,
        functionName: 'getL1Fee',
        args: [transaction],
    });
}
//# sourceMappingURL=estimateL1Fee.js.map