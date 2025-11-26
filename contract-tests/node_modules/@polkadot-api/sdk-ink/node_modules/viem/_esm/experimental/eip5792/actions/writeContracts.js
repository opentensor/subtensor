// TODO(v3): Remove this.
import { sendCalls, } from '../../../actions/wallet/sendCalls.js';
import { encodeFunctionData, } from '../../../utils/abi/encodeFunctionData.js';
import { getAction } from '../../../utils/getAction.js';
/**
 * Requests for the wallet to sign and broadcast a batch of write contract calls (transactions) to the network.
 *
 * - Docs: https://viem.sh/experimental/eip5792/writeContracts
 *
 * @param client - Client to use
 * @param parameters - {@link WriteContractsParameters}
 * @returns Unique identifier for the call batch. {@link WriteContractsReturnType}
 *
 * @example
 * import { createPublicClient, http, parseAbi } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { writeContracts } from 'viem/experimental'
 *
 * const client = createPublicClient({
 *   chain: mainnet,
 *   transport: http(),
 * })
 * const abi = parseAbi([
 *   'function approve(address, uint256) returns (bool)',
 *   'function transferFrom(address, address, uint256) returns (bool)',
 * ])
 * const id = await writeContracts(client, {
 *   contracts: [
 *     {
 *       address: '0xFBA3912Ca04dd458c843e2EE08967fC04f3579c2',
 *       abi,
 *       functionName: 'approve',
 *       args: ['0xa5cc3c03994DB5b0d9A5eEdD10CabaB0813678AC', 100n],
 *     },
 *     {
 *       address: '0xFBA3912Ca04dd458c843e2EE08967fC04f3579c2',
 *       abi,
 *       functionName: 'transferFrom',
 *       args: [
 *         '0xa5cc3c03994DB5b0d9A5eEdD10CabaB0813678AC',
 *         '0x0000000000000000000000000000000000000000',
 *         100n
 *       ],
 *     },
 *   ],
 * })
 */
export async function writeContracts(client, parameters) {
    const contracts = parameters.contracts;
    const calls = contracts.map((contract) => {
        const { address, abi, functionName, args, value } = contract;
        return {
            data: encodeFunctionData({
                abi,
                functionName,
                args,
            }),
            to: address,
            value,
        };
    });
    return getAction(client, sendCalls, 'sendCalls')({ ...parameters, calls });
}
//# sourceMappingURL=writeContracts.js.map