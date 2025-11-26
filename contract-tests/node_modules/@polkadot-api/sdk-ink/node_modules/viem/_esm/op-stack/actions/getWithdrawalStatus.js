import { readContract, } from '../../actions/public/readContract.js';
import { ContractFunctionRevertedError } from '../../errors/contract.js';
import { anchorStateRegistryAbi, portal2Abi, portalAbi } from '../abis.js';
import { ReceiptContainsNoWithdrawalsError, } from '../errors/withdrawal.js';
import { getWithdrawals, } from '../utils/getWithdrawals.js';
import { getGame, } from './getGame.js';
import { getL2Output, } from './getL2Output.js';
import { getPortalVersion, } from './getPortalVersion.js';
import { getTimeToFinalize, } from './getTimeToFinalize.js';
/**
 * Returns the current status of a withdrawal. Used for the [Withdrawal](/op-stack/guides/withdrawals) flow.
 *
 * - Docs: https://viem.sh/op-stack/actions/getWithdrawalStatus
 *
 * @param client - Client to use
 * @param parameters - {@link GetWithdrawalStatusParameters}
 * @returns Status of the withdrawal. {@link GetWithdrawalStatusReturnType}
 *
 * @example
 * import { createPublicClient, http } from 'viem'
 * import { getBlockNumber } from 'viem/actions'
 * import { mainnet, optimism } from 'viem/chains'
 * import { getWithdrawalStatus } from 'viem/op-stack'
 *
 * const publicClientL1 = createPublicClient({
 *   chain: mainnet,
 *   transport: http(),
 * })
 * const publicClientL2 = createPublicClient({
 *   chain: optimism,
 *   transport: http(),
 * })
 *
 * const receipt = await publicClientL2.getTransactionReceipt({ hash: '0x...' })
 * const status = await getWithdrawalStatus(publicClientL1, {
 *   receipt,
 *   targetChain: optimism
 * })
 */
export async function getWithdrawalStatus(client, parameters) {
    const { chain = client.chain, gameLimit = 100, receipt, targetChain: targetChain_, logIndex = 0, } = parameters;
    const targetChain = targetChain_;
    const portalAddress = (() => {
        if (parameters.portalAddress)
            return parameters.portalAddress;
        if (chain)
            return targetChain.contracts.portal[chain.id].address;
        return Object.values(targetChain.contracts.portal)[0].address;
    })();
    const l2BlockNumber = receipt?.blockNumber ?? parameters.l2BlockNumber;
    const withdrawal = (() => {
        if (receipt) {
            const withdrawal = getWithdrawals({ logs: receipt.logs })[logIndex];
            if (!withdrawal)
                throw new ReceiptContainsNoWithdrawalsError({
                    hash: receipt.transactionHash,
                });
            return withdrawal;
        }
        return {
            sender: parameters.sender,
            withdrawalHash: parameters.withdrawalHash,
        };
    })();
    const portalVersion = await getPortalVersion(client, parameters);
    // Legacy (Portal < v3)
    if (portalVersion.major < 3) {
        const [outputResult, proveResult, finalizedResult, timeToFinalizeResult] = await Promise.allSettled([
            getL2Output(client, {
                ...parameters,
                l2BlockNumber,
            }),
            readContract(client, {
                abi: portalAbi,
                address: portalAddress,
                functionName: 'provenWithdrawals',
                args: [withdrawal.withdrawalHash],
            }),
            readContract(client, {
                abi: portalAbi,
                address: portalAddress,
                functionName: 'finalizedWithdrawals',
                args: [withdrawal.withdrawalHash],
            }),
            getTimeToFinalize(client, {
                ...parameters,
                withdrawalHash: withdrawal.withdrawalHash,
            }),
        ]);
        // If the L2 Output is not processed yet (ie. the actions throws), this means
        // that the withdrawal is not ready to prove.
        if (outputResult.status === 'rejected') {
            const error = outputResult.reason;
            if (error.cause instanceof ContractFunctionRevertedError &&
                error.cause.data?.args?.[0] ===
                    'L2OutputOracle: cannot get output for a block that has not been proposed')
                return 'waiting-to-prove';
            throw error;
        }
        if (proveResult.status === 'rejected')
            throw proveResult.reason;
        if (finalizedResult.status === 'rejected')
            throw finalizedResult.reason;
        if (timeToFinalizeResult.status === 'rejected')
            throw timeToFinalizeResult.reason;
        const [_, proveTimestamp] = proveResult.value;
        if (!proveTimestamp)
            return 'ready-to-prove';
        const finalized = finalizedResult.value;
        if (finalized)
            return 'finalized';
        const { seconds } = timeToFinalizeResult.value;
        return seconds > 0 ? 'waiting-to-finalize' : 'ready-to-finalize';
    }
    const numProofSubmitters = await readContract(client, {
        abi: portal2Abi,
        address: portalAddress,
        functionName: 'numProofSubmitters',
        args: [withdrawal.withdrawalHash],
    }).catch(() => 1n);
    const proofSubmitter = await readContract(client, {
        abi: portal2Abi,
        address: portalAddress,
        functionName: 'proofSubmitters',
        args: [withdrawal.withdrawalHash, numProofSubmitters - 1n],
    }).catch(() => withdrawal.sender);
    const [disputeGameResult, provenWithdrawalsResult, checkWithdrawalResult, finalizedResult,] = await Promise.allSettled([
        getGame(client, {
            ...parameters,
            l2BlockNumber,
            limit: gameLimit,
        }),
        readContract(client, {
            abi: portal2Abi,
            address: portalAddress,
            functionName: 'provenWithdrawals',
            args: [withdrawal.withdrawalHash, proofSubmitter],
        }),
        readContract(client, {
            abi: portal2Abi,
            address: portalAddress,
            functionName: 'checkWithdrawal',
            args: [withdrawal.withdrawalHash, proofSubmitter],
        }),
        readContract(client, {
            abi: portal2Abi,
            address: portalAddress,
            functionName: 'finalizedWithdrawals',
            args: [withdrawal.withdrawalHash],
        }),
    ]);
    if (finalizedResult.status === 'fulfilled' && finalizedResult.value)
        return 'finalized';
    if (provenWithdrawalsResult.status === 'rejected')
        throw provenWithdrawalsResult.reason;
    if (disputeGameResult.status === 'rejected') {
        const error = disputeGameResult.reason;
        if (error.name === 'GameNotFoundError')
            return 'waiting-to-prove';
        throw disputeGameResult.reason;
    }
    if (checkWithdrawalResult.status === 'rejected') {
        const error = checkWithdrawalResult.reason;
        if (error.cause instanceof ContractFunctionRevertedError) {
            // All potential error causes listed here, can either be the error string or the error name
            // if custom error types are returned.
            const errorCauses = {
                'ready-to-prove': [
                    'OptimismPortal: invalid game type',
                    'OptimismPortal: withdrawal has not been proven yet',
                    'OptimismPortal: withdrawal has not been proven by proof submitter address yet',
                    'OptimismPortal: dispute game created before respected game type was updated',
                    'InvalidGameType',
                    'LegacyGame',
                    'Unproven',
                    // After U16
                    'OptimismPortal_Unproven',
                    'OptimismPortal_InvalidProofTimestamp',
                ],
                'waiting-to-finalize': [
                    'OptimismPortal: proven withdrawal has not matured yet',
                    'OptimismPortal: output proposal has not been finalized yet',
                    'OptimismPortal: output proposal in air-gap',
                ],
            };
            // Pick out the error message and/or error name
            // Return the status based on the error
            const errors = [
                error.cause.data?.errorName,
                error.cause.data?.args?.[0],
            ];
            // After U16 we get a generic error message (OptimismPortal_InvalidRootClaim) because the
            // OptimismPortal will call AnchorStateRegistry.isGameClaimValid which simply returns
            // true/false. If we get this generic error, we need to figure out why the function returned
            // false and return a proper status accordingly. We can also check these conditions when we
            // get ProofNotOldEnough so users can be notified when their pending proof becomes invalid
            // before it can be finalized.
            if (errors.includes('OptimismPortal_InvalidRootClaim') ||
                errors.includes('OptimismPortal_ProofNotOldEnough')) {
                // Get the dispute game address from the proven withdrawal.
                const disputeGameAddress = provenWithdrawalsResult.value[0];
                // Get the AnchorStateRegistry address from the portal.
                const anchorStateRegistry = await readContract(client, {
                    abi: portal2Abi,
                    address: portalAddress,
                    functionName: 'anchorStateRegistry',
                });
                // Check if the game is proper, respected, and finalized.
                const [isGameProperResult, isGameRespectedResult, isGameFinalizedResult,] = await Promise.allSettled([
                    readContract(client, {
                        abi: anchorStateRegistryAbi,
                        address: anchorStateRegistry,
                        functionName: 'isGameProper',
                        args: [disputeGameAddress],
                    }),
                    readContract(client, {
                        abi: anchorStateRegistryAbi,
                        address: anchorStateRegistry,
                        functionName: 'isGameRespected',
                        args: [disputeGameAddress],
                    }),
                    readContract(client, {
                        abi: anchorStateRegistryAbi,
                        address: anchorStateRegistry,
                        functionName: 'isGameFinalized',
                        args: [disputeGameAddress],
                    }),
                ]);
                // If any of the calls failed, throw the error.
                if (isGameProperResult.status === 'rejected')
                    throw isGameProperResult.reason;
                if (isGameRespectedResult.status === 'rejected')
                    throw isGameRespectedResult.reason;
                if (isGameFinalizedResult.status === 'rejected')
                    throw isGameFinalizedResult.reason;
                // If the game isn't proper, the user needs to re-prove.
                if (!isGameProperResult.value) {
                    return 'ready-to-prove';
                }
                // If the game isn't respected, the user needs to re-prove.
                if (!isGameRespectedResult.value) {
                    return 'ready-to-prove';
                }
                // If the game isn't finalized, the user needs to wait to finalize.
                if (!isGameFinalizedResult.value) {
                    return 'waiting-to-finalize';
                }
                // If the actual error was ProofNotOldEnough, then at this point the game is probably
                // completely fine but the proof hasn't passed the waiting period. Otherwise, the only
                // reason we'd be here is if the game resolved in favor of the challenger, which means the
                // user needs to re-prove the withdrawal.
                if (errors.includes('OptimismPortal_ProofNotOldEnough')) {
                    return 'waiting-to-finalize';
                }
                return 'ready-to-prove';
            }
            if (errorCauses['ready-to-prove'].some((cause) => errors.includes(cause)))
                return 'ready-to-prove';
            if (errorCauses['waiting-to-finalize'].some((cause) => errors.includes(cause)))
                return 'waiting-to-finalize';
        }
        throw checkWithdrawalResult.reason;
    }
    if (finalizedResult.status === 'rejected')
        throw finalizedResult.reason;
    return 'ready-to-finalize';
}
//# sourceMappingURL=getWithdrawalStatus.js.map