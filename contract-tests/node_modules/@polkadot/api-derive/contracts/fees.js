import { map, of } from 'rxjs';
import { memo } from '../util/index.js';
function queryConstants(api) {
    return of([
        // deprecated
        api.consts.contracts['callBaseFee'] || api.registry.createType('Balance'),
        api.consts.contracts['contractFee'] || api.registry.createType('Balance'),
        api.consts.contracts['creationFee'] || api.registry.createType('Balance'),
        api.consts.contracts['transactionBaseFee'] || api.registry.createType('Balance'),
        api.consts.contracts['transactionByteFee'] || api.registry.createType('Balance'),
        api.consts.contracts['transferFee'] || api.registry.createType('Balance'),
        // current
        api.consts.contracts['rentByteFee'] || api.registry.createType('Balance'),
        api.consts.contracts['rentDepositOffset'] || api.registry.createType('Balance'),
        api.consts.contracts['surchargeReward'] || api.registry.createType('Balance'),
        api.consts.contracts['tombstoneDeposit'] || api.registry.createType('Balance')
    ]);
}
/**
 * @name fees
 * @returns An object containing the combined results of the queries for
 * all relevant contract fees as declared in the substrate chain spec.
 * @example
 * ```javascript
 * api.derive.contracts.fees(([creationFee, transferFee]) => {
 *   console.log(`The fee for creating a new contract on this chain is ${creationFee} units. The fee required to call this contract is ${transferFee} units.`);
 * });
 * ```
 */
export function fees(instanceId, api) {
    return memo(instanceId, () => {
        return queryConstants(api).pipe(map(([callBaseFee, contractFee, creationFee, transactionBaseFee, transactionByteFee, transferFee, rentByteFee, rentDepositOffset, surchargeReward, tombstoneDeposit]) => ({
            callBaseFee,
            contractFee,
            creationFee,
            rentByteFee,
            rentDepositOffset,
            surchargeReward,
            tombstoneDeposit,
            transactionBaseFee,
            transactionByteFee,
            transferFee
        })));
    });
}
