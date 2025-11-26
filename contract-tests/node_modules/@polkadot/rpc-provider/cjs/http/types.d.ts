import type { Logger } from '@polkadot/util/types';
import type { RpcCoder } from '../coder/index.js';
export interface HttpState {
    coder: RpcCoder;
    endpoint: string;
    l: Logger;
}
