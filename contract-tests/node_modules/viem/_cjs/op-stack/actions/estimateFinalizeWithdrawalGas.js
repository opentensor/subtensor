"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.estimateFinalizeWithdrawalGas = estimateFinalizeWithdrawalGas;
const estimateContractGas_js_1 = require("../../actions/public/estimateContractGas.js");
const abis_js_1 = require("../abis.js");
async function estimateFinalizeWithdrawalGas(client, parameters) {
    const { account, chain = client.chain, gas, maxFeePerGas, maxPriorityFeePerGas, nonce, proofSubmitter, targetChain, withdrawal, } = parameters;
    const portalAddress = (() => {
        if (parameters.portalAddress)
            return parameters.portalAddress;
        if (chain)
            return targetChain.contracts.portal[chain.id].address;
        return Object.values(targetChain.contracts.portal)[0].address;
    })();
    const [functionName, args, abi] = proofSubmitter
        ? [
            'finalizeWithdrawalTransactionExternalProof',
            [withdrawal, proofSubmitter],
            abis_js_1.portal2Abi,
        ]
        : ['finalizeWithdrawalTransaction', [withdrawal], abis_js_1.portalAbi];
    const params = {
        account,
        abi,
        address: portalAddress,
        functionName,
        args,
        gas,
        maxFeePerGas,
        maxPriorityFeePerGas,
        nonce,
        chain,
    };
    return (0, estimateContractGas_js_1.estimateContractGas)(client, params);
}
//# sourceMappingURL=estimateFinalizeWithdrawalGas.js.map