import type { Observable } from 'rxjs';
import type { BN } from '@polkadot/util';
import type { DeriveApi } from '../types.js';
import type { Bag } from './types.js';
export declare function _getIds(instanceId: string, api: DeriveApi): (ids: (BN | number)[]) => Observable<Bag[]>;
export declare function all(instanceId: string, api: DeriveApi): () => Observable<Bag[]>;
/**
 * @name get
 * @param {(BN | number)} id The id of the bag to retrieve.
 * @description Retrieves a specific bag from the BagsList pallet by its id.
 */
export declare function get(instanceId: string, api: DeriveApi): (id: BN | number) => Observable<Bag>;
