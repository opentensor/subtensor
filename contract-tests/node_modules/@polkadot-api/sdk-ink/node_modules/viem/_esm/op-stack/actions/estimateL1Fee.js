import { readContract, } from '../../actions/public/readContract.js';
import { getChainContractAddress } from '../../utils/chain/getChainContractAddress.js';
import { serializeTransaction, } from '../../utils/transaction/serializeTransaction.js';
import { parseGwei } from '../../utils/unit/parseGwei.js';
import { gasPriceOracleAbi } from '../abis.js';
import { contracts } from '../contracts.js';
/**
 * Estimates the L1 data fee required to execute an L2 transaction.
 *
 * @param client - Client to use
 * @param parameters - {@link EstimateL1FeeParameters}
 * @returns The fee (in wei). {@link EstimateL1FeeReturnType}
 *
 * @example
 * import { createPublicClient, http, parseEther } from 'viem'
 * import { optimism } from 'viem/chains'
 * import { estimateL1Fee } from 'viem/chains/optimism'
 *
 * const client = createPublicClient({
 *   chain: optimism,
 *   transport: http(),
 * })
 * const l1Fee = await estimateL1Fee(client, {
 *   account: '0xA0Cf798816D4b9b9866b5330EEa46a18382f251e',
 *   to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *   value: parseEther('1'),
 * })
 */
export async function estimateL1Fee(client, args) {
    const { chain = client.chain, gasPriceOracleAddress: gasPriceOracleAddress_, } = args;
    const gasPriceOracleAddress = (() => {
        if (gasPriceOracleAddress_)
            return gasPriceOracleAddress_;
        if (chain)
            return getChainContractAddress({
                chain,
                contract: 'gasPriceOracle',
            });
        return contracts.gasPriceOracle.address;
    })();
    const transaction = serializeTransaction({
        ...args,
        chainId: chain?.id ?? 1,
        type: 'eip1559',
        // Set upper-limit-ish stub values. Shouldn't affect the estimate too much as we are
        // tweaking dust bytes here (as opposed to long `data` bytes).
        // See: https://github.com/ethereum-optimism/optimism/blob/54d02df55523c9e1b4b38ed082c12a42087323a0/packages/contracts-bedrock/src/L2/GasPriceOracle.sol#L242-L248.
        gas: args.data ? 300000n : 21000n,
        maxFeePerGas: parseGwei('5'),
        maxPriorityFeePerGas: parseGwei('1'),
        nonce: 1,
    });
    return readContract(client, {
        abi: gasPriceOracleAbi,
        address: gasPriceOracleAddress,
        functionName: 'getL1Fee',
        args: [transaction],
    });
}
//# sourceMappingURL=estimateL1Fee.js.map