import type { Account } from '../../../accounts/index.js';
import type { Client } from '../../../clients/createClient.js';
import type { Transport } from '../../../clients/transports/createTransport.js';
import type { Chain } from '../../../types/chain.js';
import type { Hash } from '../../../types/misc.js';
import type { ZksyncL2ToL1Log } from '../../types/log.js';
export type GetWithdrawalL2ToL1LogParameters = {
    /** Hash of the L2 transaction where the withdrawal was initiated. */
    hash: Hash;
    /** In case there were multiple withdrawals in one transaction, you may pass an index of the
     withdrawal you want to finalize. */
    index?: number | undefined;
};
export type GetWithdrawalL2ToL1LogReturnType = {
    l2ToL1LogIndex: number | null;
    l2ToL1Log: ZksyncL2ToL1Log;
};
/** @internal */
export declare function getWithdrawalL2ToL1Log<chain extends Chain | undefined, account extends Account | undefined>(client: Client<Transport, chain, account>, parameters: GetWithdrawalL2ToL1LogParameters): Promise<GetWithdrawalL2ToL1LogReturnType>;
//# sourceMappingURL=getWithdrawalL2ToL1Log.d.ts.map