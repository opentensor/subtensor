import type { Client } from '../../clients/createClient.js';
import type { Transport } from '../../clients/transports/createTransport.js';
import type { Account } from '../../types/account.js';
import type { ChainEIP712 } from '../types/chain.js';
import type { PublicZksyncRpcSchema } from '../types/eip1193.js';
export type GetGasPerPubdataReturnType = bigint;
export declare function getGasPerPubdata<chain extends ChainEIP712 | undefined, account extends Account | undefined>(client: Client<Transport, chain, account, PublicZksyncRpcSchema>): Promise<GetGasPerPubdataReturnType>;
//# sourceMappingURL=getGasPerPubdata.d.ts.map