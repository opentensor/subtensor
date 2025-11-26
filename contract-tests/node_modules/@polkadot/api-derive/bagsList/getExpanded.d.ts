import type { Observable } from 'rxjs';
import type { BN } from '@polkadot/util';
import type { DeriveApi } from '../types.js';
import type { Bag, BagExpanded } from './types.js';
/**
 * @name expand
 * @description Expands a given bag by retrieving all its nodes (accounts contained within the bag).
 * @param {Bag} bag The bag to be expanded.
 */
export declare function expand(instanceId: string, api: DeriveApi): (bag: Bag) => Observable<BagExpanded>;
/**
 * @name getExpanded
 * @description Retrieves and expands a specific bag from the BagsList pallet.
 * @param {BN | number} id The id of the bag to expand.
 */
export declare function getExpanded(instanceId: string, api: DeriveApi): (id: BN | number) => Observable<BagExpanded>;
