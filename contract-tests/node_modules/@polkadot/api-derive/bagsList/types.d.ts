import type { PalletBagsListListBag, PalletBagsListListNode } from '@polkadot/types/lookup';
import type { BN } from '@polkadot/util';
export interface Bag {
    bag: PalletBagsListListBag | null;
    bagUpper: BN;
    bagLower: BN;
    id: BN;
    index: number;
    key: string;
}
export interface BagExpanded extends Bag {
    nodes: PalletBagsListListNode[];
}
