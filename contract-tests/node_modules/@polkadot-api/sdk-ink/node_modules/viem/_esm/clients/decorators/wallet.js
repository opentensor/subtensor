import { getChainId, } from '../../actions/public/getChainId.js';
import { addChain, } from '../../actions/wallet/addChain.js';
import { deployContract, } from '../../actions/wallet/deployContract.js';
import { getAddresses, } from '../../actions/wallet/getAddresses.js';
import { getCallsStatus, } from '../../actions/wallet/getCallsStatus.js';
import { getCapabilities, } from '../../actions/wallet/getCapabilities.js';
import { getPermissions, } from '../../actions/wallet/getPermissions.js';
import { prepareAuthorization, } from '../../actions/wallet/prepareAuthorization.js';
import { prepareTransactionRequest, } from '../../actions/wallet/prepareTransactionRequest.js';
import { requestAddresses, } from '../../actions/wallet/requestAddresses.js';
import { requestPermissions, } from '../../actions/wallet/requestPermissions.js';
import { sendCalls, } from '../../actions/wallet/sendCalls.js';
import { sendCallsSync, } from '../../actions/wallet/sendCallsSync.js';
import { sendRawTransaction, } from '../../actions/wallet/sendRawTransaction.js';
import { sendRawTransactionSync, } from '../../actions/wallet/sendRawTransactionSync.js';
import { sendTransaction, } from '../../actions/wallet/sendTransaction.js';
import { sendTransactionSync, } from '../../actions/wallet/sendTransactionSync.js';
import { showCallsStatus, } from '../../actions/wallet/showCallsStatus.js';
import { signAuthorization, } from '../../actions/wallet/signAuthorization.js';
import { signMessage, } from '../../actions/wallet/signMessage.js';
import { signTransaction, } from '../../actions/wallet/signTransaction.js';
import { signTypedData, } from '../../actions/wallet/signTypedData.js';
import { switchChain, } from '../../actions/wallet/switchChain.js';
import { waitForCallsStatus, } from '../../actions/wallet/waitForCallsStatus.js';
import { watchAsset, } from '../../actions/wallet/watchAsset.js';
import { writeContract, } from '../../actions/wallet/writeContract.js';
import { writeContractSync, } from '../../actions/wallet/writeContractSync.js';
export function walletActions(client) {
    return {
        addChain: (args) => addChain(client, args),
        deployContract: (args) => deployContract(client, args),
        getAddresses: () => getAddresses(client),
        getCallsStatus: (args) => getCallsStatus(client, args),
        getCapabilities: (args) => getCapabilities(client, args),
        getChainId: () => getChainId(client),
        getPermissions: () => getPermissions(client),
        prepareAuthorization: (args) => prepareAuthorization(client, args),
        prepareTransactionRequest: (args) => prepareTransactionRequest(client, args),
        requestAddresses: () => requestAddresses(client),
        requestPermissions: (args) => requestPermissions(client, args),
        sendCalls: (args) => sendCalls(client, args),
        sendCallsSync: (args) => sendCallsSync(client, args),
        sendRawTransaction: (args) => sendRawTransaction(client, args),
        sendRawTransactionSync: (args) => sendRawTransactionSync(client, args),
        sendTransaction: (args) => sendTransaction(client, args),
        sendTransactionSync: (args) => sendTransactionSync(client, args),
        showCallsStatus: (args) => showCallsStatus(client, args),
        signAuthorization: (args) => signAuthorization(client, args),
        signMessage: (args) => signMessage(client, args),
        signTransaction: (args) => signTransaction(client, args),
        signTypedData: (args) => signTypedData(client, args),
        switchChain: (args) => switchChain(client, args),
        waitForCallsStatus: (args) => waitForCallsStatus(client, args),
        watchAsset: (args) => watchAsset(client, args),
        writeContract: (args) => writeContract(client, args),
        writeContractSync: (args) => writeContractSync(client, args),
    };
}
//# sourceMappingURL=wallet.js.map