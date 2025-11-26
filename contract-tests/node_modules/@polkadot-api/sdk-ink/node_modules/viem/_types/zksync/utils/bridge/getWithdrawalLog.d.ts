import type { Account } from '../../../accounts/types.js';
import type { Client } from '../../../clients/createClient.js';
import type { Transport } from '../../../clients/transports/createTransport.js';
import type { Chain } from '../../../types/chain.js';
import type { Hash } from '../../../types/misc.js';
import type { ZksyncLog } from '../../types/log.js';
export type GetWithdrawalLogParameters = {
    /** Hash of the L2 transaction where the withdrawal was initiated. */
    hash: Hash;
    /** In case there were multiple withdrawals in one transaction, you may pass an index of the
       withdrawal you want to finalize. */
    index?: number | undefined;
};
export type GetWithdrawalLogReturnType = {
    log: ZksyncLog;
    l1BatchTxId: bigint | null;
};
/** @internal */
export declare function getWithdrawalLog<chain extends Chain | undefined, account extends Account | undefined>(client: Client<Transport, chain, account>, parameters: GetWithdrawalLogParameters): Promise<GetWithdrawalLogReturnType>;
//# sourceMappingURL=getWithdrawalLog.d.ts.map