// TODO(v3): Remove this.
import { getCallsStatus, } from '../../../actions/wallet/getCallsStatus.js';
import { getCapabilities, } from '../../../actions/wallet/getCapabilities.js';
import { sendCalls, } from '../../../actions/wallet/sendCalls.js';
import { showCallsStatus, } from '../../../actions/wallet/showCallsStatus.js';
import { waitForCallsStatus, } from '../../../actions/wallet/waitForCallsStatus.js';
import { writeContracts, } from '../actions/writeContracts.js';
/**
 * A suite of EIP-5792 Wallet Actions.
 *
 * - Docs: https://viem.sh/experimental
 *
 * @example
 * import { createPublicClient, createWalletClient, http } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { eip5792Actions } from 'viem/experimental'
 *
 * const walletClient = createWalletClient({
 *   chain: mainnet,
 *   transport: http(),
 * }).extend(eip5792Actions())
 *
 * const hash = await walletClient.sendCalls({...})
 */
export function eip5792Actions() {
    return (client) => {
        return {
            getCallsStatus: (parameters) => getCallsStatus(client, parameters),
            getCapabilities: ((parameters) => getCapabilities(client, parameters)),
            sendCalls: (parameters) => sendCalls(client, parameters),
            showCallsStatus: (parameters) => showCallsStatus(client, parameters),
            waitForCallsStatus: (parameters) => waitForCallsStatus(client, parameters),
            writeContracts: (parameters) => writeContracts(client, parameters),
        };
    };
}
//# sourceMappingURL=eip5792.js.map