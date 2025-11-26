import { parseAccount } from '../../accounts/utils/parseAccount.js';
import { BaseError } from '../../errors/base.js';
import { AtomicityNotSupportedError, UnsupportedNonOptionalCapabilityError, } from '../../errors/rpc.js';
import { encodeFunctionData } from '../../utils/abi/encodeFunctionData.js';
import { concat } from '../../utils/data/concat.js';
import { hexToBigInt } from '../../utils/encoding/fromHex.js';
import { numberToHex } from '../../utils/encoding/toHex.js';
import { getTransactionError } from '../../utils/errors/getTransactionError.js';
import { sendTransaction } from './sendTransaction.js';
export const fallbackMagicIdentifier = '0x5792579257925792579257925792579257925792579257925792579257925792';
export const fallbackTransactionErrorMagicIdentifier = numberToHex(0, {
    size: 32,
});
/**
 * Requests the connected wallet to send a batch of calls.
 *
 * - Docs: https://viem.sh/docs/actions/wallet/sendCalls
 * - JSON-RPC Methods: [`wallet_sendCalls`](https://eips.ethereum.org/EIPS/eip-5792)
 *
 * @param client - Client to use
 * @returns Transaction identifier. {@link SendCallsReturnType}
 *
 * @example
 * import { createWalletClient, custom } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { sendCalls } from 'viem/actions'
 *
 * const client = createWalletClient({
 *   chain: mainnet,
 *   transport: custom(window.ethereum),
 * })
 * const id = await sendCalls(client, {
 *   account: '0xA0Cf798816D4b9b9866b5330EEa46a18382f251e',
 *   calls: [
 *     {
 *       data: '0xdeadbeef',
 *       to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *     },
 *     {
 *       to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *       value: 69420n,
 *     },
 *   ],
 * })
 */
export async function sendCalls(client, parameters) {
    const { account: account_ = client.account, capabilities, chain = client.chain, experimental_fallback, experimental_fallbackDelay = 32, forceAtomic = false, id, version = '2.0.0', } = parameters;
    const account = account_ ? parseAccount(account_) : null;
    const calls = parameters.calls.map((call_) => {
        const call = call_;
        const data = call.abi
            ? encodeFunctionData({
                abi: call.abi,
                functionName: call.functionName,
                args: call.args,
            })
            : call.data;
        return {
            data: call.dataSuffix && data ? concat([data, call.dataSuffix]) : data,
            to: call.to,
            value: call.value ? numberToHex(call.value) : undefined,
        };
    });
    try {
        const response = await client.request({
            method: 'wallet_sendCalls',
            params: [
                {
                    atomicRequired: forceAtomic,
                    calls,
                    capabilities,
                    chainId: numberToHex(chain.id),
                    from: account?.address,
                    id,
                    version,
                },
            ],
        }, { retryCount: 0 });
        if (typeof response === 'string')
            return { id: response };
        return response;
    }
    catch (err) {
        const error = err;
        // If the transport does not support EIP-5792, fall back to
        // `eth_sendTransaction`.
        if (experimental_fallback &&
            (error.name === 'MethodNotFoundRpcError' ||
                error.name === 'MethodNotSupportedRpcError' ||
                error.name === 'UnknownRpcError' ||
                error.details
                    .toLowerCase()
                    .includes('does not exist / is not available') ||
                error.details.toLowerCase().includes('missing or invalid. request()') ||
                error.details
                    .toLowerCase()
                    .includes('did not match any variant of untagged enum') ||
                error.details
                    .toLowerCase()
                    .includes('account upgraded to unsupported contract') ||
                error.details.toLowerCase().includes('eip-7702 not supported') ||
                error.details.toLowerCase().includes('unsupported wc_ method') ||
                // magic.link
                error.details
                    .toLowerCase()
                    .includes('feature toggled misconfigured') ||
                // Trust Wallet
                error.details
                    .toLowerCase()
                    .includes('jsonrpcengine: response has no error or result for request'))) {
            if (capabilities) {
                const hasNonOptionalCapability = Object.values(capabilities).some((capability) => !capability.optional);
                if (hasNonOptionalCapability) {
                    const message = 'non-optional `capabilities` are not supported on fallback to `eth_sendTransaction`.';
                    throw new UnsupportedNonOptionalCapabilityError(new BaseError(message, {
                        details: message,
                    }));
                }
            }
            if (forceAtomic && calls.length > 1) {
                const message = '`forceAtomic` is not supported on fallback to `eth_sendTransaction`.';
                throw new AtomicityNotSupportedError(new BaseError(message, {
                    details: message,
                }));
            }
            const promises = [];
            for (const call of calls) {
                const promise = sendTransaction(client, {
                    account,
                    chain,
                    data: call.data,
                    to: call.to,
                    value: call.value ? hexToBigInt(call.value) : undefined,
                });
                promises.push(promise);
                // Note: some browser wallets require a small delay between transactions
                // to prevent duplicate JSON-RPC requests.
                if (experimental_fallbackDelay > 0)
                    await new Promise((resolve) => setTimeout(resolve, experimental_fallbackDelay));
            }
            const results = await Promise.allSettled(promises);
            if (results.every((r) => r.status === 'rejected'))
                throw results[0].reason;
            const hashes = results.map((result) => {
                if (result.status === 'fulfilled')
                    return result.value;
                return fallbackTransactionErrorMagicIdentifier;
            });
            return {
                id: concat([
                    ...hashes,
                    numberToHex(chain.id, { size: 32 }),
                    fallbackMagicIdentifier,
                ]),
            };
        }
        throw getTransactionError(err, {
            ...parameters,
            account,
            chain: parameters.chain,
        });
    }
}
//# sourceMappingURL=sendCalls.js.map