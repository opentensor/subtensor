import type { Observable } from 'rxjs';
import type { PalletBagsListListBag, PalletBagsListListNode } from '@polkadot/types/lookup';
import type { DeriveApi } from '../types.js';
/**
 * @name listNodes
 * @param {(PalletBagsListListBag | null)} bag A reference to a specific bag in the BagsList pallet.
 * @description Retrieves the list of nodes (accounts) contained in a specific bag within the BagsList pallet.
 */
export declare function listNodes(instanceId: string, api: DeriveApi): (bag: PalletBagsListListBag | null) => Observable<PalletBagsListListNode[]>;
