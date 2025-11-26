import type { HexString } from '@polkadot/util/types';
export interface Check {
    data: HexString;
    fails?: string[];
}
