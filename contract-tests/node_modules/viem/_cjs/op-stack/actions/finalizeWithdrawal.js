"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.finalizeWithdrawal = finalizeWithdrawal;
const writeContract_js_1 = require("../../actions/wallet/writeContract.js");
const abis_js_1 = require("../abis.js");
const estimateFinalizeWithdrawalGas_js_1 = require("./estimateFinalizeWithdrawalGas.js");
async function finalizeWithdrawal(client, parameters) {
    const { account, chain = client.chain, gas, maxFeePerGas, maxPriorityFeePerGas, nonce, proofSubmitter, targetChain, withdrawal, } = parameters;
    const portalAddress = (() => {
        if (parameters.portalAddress)
            return parameters.portalAddress;
        if (chain)
            return targetChain.contracts.portal[chain.id].address;
        return Object.values(targetChain.contracts.portal)[0].address;
    })();
    const gas_ = typeof gas !== 'number' && gas !== null
        ? await (0, estimateFinalizeWithdrawalGas_js_1.estimateFinalizeWithdrawalGas)(client, parameters)
        : undefined;
    const [functionName, args, abi] = proofSubmitter
        ? [
            'finalizeWithdrawalTransactionExternalProof',
            [withdrawal, proofSubmitter],
            abis_js_1.portal2Abi,
        ]
        : ['finalizeWithdrawalTransaction', [withdrawal], abis_js_1.portalAbi];
    return (0, writeContract_js_1.writeContract)(client, {
        account: account,
        abi,
        address: portalAddress,
        chain,
        functionName,
        args,
        gas: gas_,
        maxFeePerGas,
        maxPriorityFeePerGas,
        nonce,
    });
}
//# sourceMappingURL=finalizeWithdrawal.js.map