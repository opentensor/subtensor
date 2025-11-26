import { parseAccount, } from '../../accounts/utils/parseAccount.js';
import { numberToHex, } from '../../utils/encoding/toHex.js';
import { getCallError, } from '../../utils/errors/getCallError.js';
import { extract } from '../../utils/formatters/extract.js';
import { formatTransactionRequest, } from '../../utils/formatters/transactionRequest.js';
import { assertRequest } from '../../utils/transaction/assertRequest.js';
/**
 * Creates an EIP-2930 access list.
 *
 * - Docs: https://viem.sh/docs/actions/public/createAccessList
 * - JSON-RPC Methods: `eth_createAccessList`
 *
 * @param client - Client to use
 * @param parameters - {@link CreateAccessListParameters}
 * @returns The access list. {@link CreateAccessListReturnType}
 *
 * @example
 * import { createPublicClient, http } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { createAccessList } from 'viem/public'
 *
 * const client = createPublicClient({
 *   chain: mainnet,
 *   transport: http(),
 * })
 * const data = await createAccessList(client, {
 *   account: '0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266',
 *   data: '0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2',
 *   to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 * })
 */
export async function createAccessList(client, args) {
    const { account: account_ = client.account, blockNumber, blockTag = 'latest', blobs, data, gas, gasPrice, maxFeePerBlobGas, maxFeePerGas, maxPriorityFeePerGas, to, value, ...rest } = args;
    const account = account_ ? parseAccount(account_) : undefined;
    try {
        assertRequest(args);
        const blockNumberHex = typeof blockNumber === 'bigint' ? numberToHex(blockNumber) : undefined;
        const block = blockNumberHex || blockTag;
        const chainFormat = client.chain?.formatters?.transactionRequest?.format;
        const format = chainFormat || formatTransactionRequest;
        const request = format({
            // Pick out extra data that might exist on the chain's transaction request type.
            ...extract(rest, { format: chainFormat }),
            account,
            blobs,
            data,
            gas,
            gasPrice,
            maxFeePerBlobGas,
            maxFeePerGas,
            maxPriorityFeePerGas,
            to,
            value,
        }, 'createAccessList');
        const response = await client.request({
            method: 'eth_createAccessList',
            params: [request, block],
        });
        return {
            accessList: response.accessList,
            gasUsed: BigInt(response.gasUsed),
        };
    }
    catch (err) {
        throw getCallError(err, {
            ...args,
            account,
            chain: client.chain,
        });
    }
}
//# sourceMappingURL=createAccessList.js.map