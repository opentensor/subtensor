import { sliceHex } from '../../utils/data/slice.js';
import { trim } from '../../utils/data/trim.js';
import { hexToBigInt, hexToNumber } from '../../utils/encoding/fromHex.js';
import { receiptStatuses } from '../../utils/formatters/transactionReceipt.js';
import { fallbackMagicIdentifier, fallbackTransactionErrorMagicIdentifier, } from './sendCalls.js';
/**
 * Returns the status of a call batch that was sent via `sendCalls`.
 *
 * - Docs: https://viem.sh/docs/actions/wallet/getCallsStatus
 * - JSON-RPC Methods: [`wallet_getCallsStatus`](https://eips.ethereum.org/EIPS/eip-5792)
 *
 * @param client - Client to use
 * @returns Status of the calls. {@link GetCallsStatusReturnType}
 *
 * @example
 * import { createWalletClient, custom } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { getCallsStatus } from 'viem/actions'
 *
 * const client = createWalletClient({
 *   chain: mainnet,
 *   transport: custom(window.ethereum),
 * })
 * const { receipts, status } = await getCallsStatus(client, { id: '0xdeadbeef' })
 */
export async function getCallsStatus(client, parameters) {
    async function getStatus(id) {
        const isTransactions = id.endsWith(fallbackMagicIdentifier.slice(2));
        if (isTransactions) {
            const chainId = trim(sliceHex(id, -64, -32));
            const hashes = sliceHex(id, 0, -64)
                .slice(2)
                .match(/.{1,64}/g);
            const receipts = await Promise.all(hashes.map((hash) => fallbackTransactionErrorMagicIdentifier.slice(2) !== hash
                ? client.request({
                    method: 'eth_getTransactionReceipt',
                    params: [`0x${hash}`],
                }, { dedupe: true })
                : undefined));
            const status = (() => {
                if (receipts.some((r) => r === null))
                    return 100; // pending
                if (receipts.every((r) => r?.status === '0x1'))
                    return 200; // success
                if (receipts.every((r) => r?.status === '0x0'))
                    return 500; // complete failure
                return 600; // partial failure
            })();
            return {
                atomic: false,
                chainId: hexToNumber(chainId),
                receipts: receipts.filter(Boolean),
                status,
                version: '2.0.0',
            };
        }
        return client.request({
            method: 'wallet_getCallsStatus',
            params: [id],
        });
    }
    const { atomic = false, chainId, receipts, version = '2.0.0', ...response } = await getStatus(parameters.id);
    const [status, statusCode] = (() => {
        const statusCode = response.status;
        if (statusCode >= 100 && statusCode < 200)
            return ['pending', statusCode];
        if (statusCode >= 200 && statusCode < 300)
            return ['success', statusCode];
        if (statusCode >= 300 && statusCode < 700)
            return ['failure', statusCode];
        // @ts-expect-error: for backwards compatibility
        if (statusCode === 'CONFIRMED')
            return ['success', 200];
        // @ts-expect-error: for backwards compatibility
        if (statusCode === 'PENDING')
            return ['pending', 100];
        return [undefined, statusCode];
    })();
    return {
        ...response,
        atomic,
        // @ts-expect-error: for backwards compatibility
        chainId: chainId ? hexToNumber(chainId) : undefined,
        receipts: receipts?.map((receipt) => ({
            ...receipt,
            blockNumber: hexToBigInt(receipt.blockNumber),
            gasUsed: hexToBigInt(receipt.gasUsed),
            status: receiptStatuses[receipt.status],
        })) ?? [],
        statusCode,
        status,
        version,
    };
}
//# sourceMappingURL=getCallsStatus.js.map