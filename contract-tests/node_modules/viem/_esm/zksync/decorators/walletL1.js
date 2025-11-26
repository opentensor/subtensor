import { finalizeWithdrawal, } from '../actions/finalizeWithdrawal.js';
import { requestExecute, } from '../actions/requestExecute.js';
export function walletActionsL1() {
    return (client) => ({
        finalizeWithdrawal: (args) => finalizeWithdrawal(client, args),
        requestExecute: (args) => requestExecute(client, args),
    });
}
//# sourceMappingURL=walletL1.js.map