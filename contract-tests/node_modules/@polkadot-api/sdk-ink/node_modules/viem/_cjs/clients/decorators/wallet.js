"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.walletActions = walletActions;
const getChainId_js_1 = require("../../actions/public/getChainId.js");
const addChain_js_1 = require("../../actions/wallet/addChain.js");
const deployContract_js_1 = require("../../actions/wallet/deployContract.js");
const getAddresses_js_1 = require("../../actions/wallet/getAddresses.js");
const getCallsStatus_js_1 = require("../../actions/wallet/getCallsStatus.js");
const getCapabilities_js_1 = require("../../actions/wallet/getCapabilities.js");
const getPermissions_js_1 = require("../../actions/wallet/getPermissions.js");
const prepareAuthorization_js_1 = require("../../actions/wallet/prepareAuthorization.js");
const prepareTransactionRequest_js_1 = require("../../actions/wallet/prepareTransactionRequest.js");
const requestAddresses_js_1 = require("../../actions/wallet/requestAddresses.js");
const requestPermissions_js_1 = require("../../actions/wallet/requestPermissions.js");
const sendCalls_js_1 = require("../../actions/wallet/sendCalls.js");
const sendCallsSync_js_1 = require("../../actions/wallet/sendCallsSync.js");
const sendRawTransaction_js_1 = require("../../actions/wallet/sendRawTransaction.js");
const sendRawTransactionSync_js_1 = require("../../actions/wallet/sendRawTransactionSync.js");
const sendTransaction_js_1 = require("../../actions/wallet/sendTransaction.js");
const sendTransactionSync_js_1 = require("../../actions/wallet/sendTransactionSync.js");
const showCallsStatus_js_1 = require("../../actions/wallet/showCallsStatus.js");
const signAuthorization_js_1 = require("../../actions/wallet/signAuthorization.js");
const signMessage_js_1 = require("../../actions/wallet/signMessage.js");
const signTransaction_js_1 = require("../../actions/wallet/signTransaction.js");
const signTypedData_js_1 = require("../../actions/wallet/signTypedData.js");
const switchChain_js_1 = require("../../actions/wallet/switchChain.js");
const waitForCallsStatus_js_1 = require("../../actions/wallet/waitForCallsStatus.js");
const watchAsset_js_1 = require("../../actions/wallet/watchAsset.js");
const writeContract_js_1 = require("../../actions/wallet/writeContract.js");
const writeContractSync_js_1 = require("../../actions/wallet/writeContractSync.js");
function walletActions(client) {
    return {
        addChain: (args) => (0, addChain_js_1.addChain)(client, args),
        deployContract: (args) => (0, deployContract_js_1.deployContract)(client, args),
        getAddresses: () => (0, getAddresses_js_1.getAddresses)(client),
        getCallsStatus: (args) => (0, getCallsStatus_js_1.getCallsStatus)(client, args),
        getCapabilities: (args) => (0, getCapabilities_js_1.getCapabilities)(client, args),
        getChainId: () => (0, getChainId_js_1.getChainId)(client),
        getPermissions: () => (0, getPermissions_js_1.getPermissions)(client),
        prepareAuthorization: (args) => (0, prepareAuthorization_js_1.prepareAuthorization)(client, args),
        prepareTransactionRequest: (args) => (0, prepareTransactionRequest_js_1.prepareTransactionRequest)(client, args),
        requestAddresses: () => (0, requestAddresses_js_1.requestAddresses)(client),
        requestPermissions: (args) => (0, requestPermissions_js_1.requestPermissions)(client, args),
        sendCalls: (args) => (0, sendCalls_js_1.sendCalls)(client, args),
        sendCallsSync: (args) => (0, sendCallsSync_js_1.sendCallsSync)(client, args),
        sendRawTransaction: (args) => (0, sendRawTransaction_js_1.sendRawTransaction)(client, args),
        sendRawTransactionSync: (args) => (0, sendRawTransactionSync_js_1.sendRawTransactionSync)(client, args),
        sendTransaction: (args) => (0, sendTransaction_js_1.sendTransaction)(client, args),
        sendTransactionSync: (args) => (0, sendTransactionSync_js_1.sendTransactionSync)(client, args),
        showCallsStatus: (args) => (0, showCallsStatus_js_1.showCallsStatus)(client, args),
        signAuthorization: (args) => (0, signAuthorization_js_1.signAuthorization)(client, args),
        signMessage: (args) => (0, signMessage_js_1.signMessage)(client, args),
        signTransaction: (args) => (0, signTransaction_js_1.signTransaction)(client, args),
        signTypedData: (args) => (0, signTypedData_js_1.signTypedData)(client, args),
        switchChain: (args) => (0, switchChain_js_1.switchChain)(client, args),
        waitForCallsStatus: (args) => (0, waitForCallsStatus_js_1.waitForCallsStatus)(client, args),
        watchAsset: (args) => (0, watchAsset_js_1.watchAsset)(client, args),
        writeContract: (args) => (0, writeContract_js_1.writeContract)(client, args),
        writeContractSync: (args) => (0, writeContractSync_js_1.writeContractSync)(client, args),
    };
}
//# sourceMappingURL=wallet.js.map