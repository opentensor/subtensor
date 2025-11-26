import { withdraw, } from '../actions/withdraw.js';
export function walletActionsL2() {
    return (client) => ({
        withdraw: (args) => withdraw(client, args),
    });
}
//# sourceMappingURL=walletL2.js.map