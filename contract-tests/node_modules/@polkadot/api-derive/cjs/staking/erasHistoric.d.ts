import type { Observable } from 'rxjs';
import type { EraIndex } from '@polkadot/types/interfaces';
import type { DeriveApi } from '../types.js';
/**
 * @name erasHistoric
 * @param {boolean} withActive? (Optional) Whether to include the active era in the result.
 */
export declare function erasHistoric(instanceId: string, api: DeriveApi): (withActive?: boolean) => Observable<EraIndex[]>;
