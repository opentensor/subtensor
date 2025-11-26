import { parseAccount, } from '../../accounts/utils/parseAccount.js';
import { BaseError } from '../../errors/base.js';
import { recoverAuthorizationAddress, } from '../../utils/authorization/recoverAuthorizationAddress.js';
import { numberToHex, } from '../../utils/encoding/toHex.js';
import { getEstimateGasError, } from '../../utils/errors/getEstimateGasError.js';
import { extract } from '../../utils/formatters/extract.js';
import { formatTransactionRequest, } from '../../utils/formatters/transactionRequest.js';
import { serializeStateOverride } from '../../utils/stateOverride.js';
import { assertRequest, } from '../../utils/transaction/assertRequest.js';
import { prepareTransactionRequest, } from '../wallet/prepareTransactionRequest.js';
/**
 * Estimates the gas necessary to complete a transaction without submitting it to the network.
 *
 * - Docs: https://viem.sh/docs/actions/public/estimateGas
 * - JSON-RPC Methods: [`eth_estimateGas`](https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_estimategas)
 *
 * @param client - Client to use
 * @param parameters - {@link EstimateGasParameters}
 * @returns The gas estimate (in gas units). {@link EstimateGasReturnType}
 *
 * @example
 * import { createPublicClient, http, parseEther } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { estimateGas } from 'viem/public'
 *
 * const client = createPublicClient({
 *   chain: mainnet,
 *   transport: http(),
 * })
 * const gasEstimate = await estimateGas(client, {
 *   account: '0xA0Cf798816D4b9b9866b5330EEa46a18382f251e',
 *   to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *   value: parseEther('1'),
 * })
 */
export async function estimateGas(client, args) {
    const { account: account_ = client.account, prepare = true } = args;
    const account = account_ ? parseAccount(account_) : undefined;
    const parameters = (() => {
        if (Array.isArray(prepare))
            return prepare;
        // Some RPC Providers do not compute versioned hashes from blobs. We will need
        // to compute them.
        if (account?.type !== 'local')
            return ['blobVersionedHashes'];
        return undefined;
    })();
    try {
        const { accessList, authorizationList, blobs, blobVersionedHashes, blockNumber, blockTag, data, gas, gasPrice, maxFeePerBlobGas, maxFeePerGas, maxPriorityFeePerGas, nonce, value, stateOverride, ...rest } = prepare
            ? (await prepareTransactionRequest(client, {
                ...args,
                parameters,
            }))
            : args;
        const blockNumberHex = typeof blockNumber === 'bigint' ? numberToHex(blockNumber) : undefined;
        const block = blockNumberHex || blockTag;
        const rpcStateOverride = serializeStateOverride(stateOverride);
        const to = await (async () => {
            // If `to` exists on the parameters, use that.
            if (rest.to)
                return rest.to;
            // If no `to` exists, and we are sending a EIP-7702 transaction, use the
            // address of the first authorization in the list.
            if (authorizationList && authorizationList.length > 0)
                return await recoverAuthorizationAddress({
                    authorization: authorizationList[0],
                }).catch(() => {
                    throw new BaseError('`to` is required. Could not infer from `authorizationList`');
                });
            // Otherwise, we are sending a deployment transaction.
            return undefined;
        })();
        assertRequest(args);
        const chainFormat = client.chain?.formatters?.transactionRequest?.format;
        const format = chainFormat || formatTransactionRequest;
        const request = format({
            // Pick out extra data that might exist on the chain's transaction request type.
            ...extract(rest, { format: chainFormat }),
            account,
            accessList,
            authorizationList,
            blobs,
            blobVersionedHashes,
            data,
            gas,
            gasPrice,
            maxFeePerBlobGas,
            maxFeePerGas,
            maxPriorityFeePerGas,
            nonce,
            to,
            value,
        }, 'estimateGas');
        return BigInt(await client.request({
            method: 'eth_estimateGas',
            params: rpcStateOverride
                ? [
                    request,
                    block ?? client.experimental_blockTag ?? 'latest',
                    rpcStateOverride,
                ]
                : block
                    ? [request, block]
                    : [request],
        }));
    }
    catch (err) {
        throw getEstimateGasError(err, {
            ...args,
            account,
            chain: client.chain,
        });
    }
}
//# sourceMappingURL=estimateGas.js.map