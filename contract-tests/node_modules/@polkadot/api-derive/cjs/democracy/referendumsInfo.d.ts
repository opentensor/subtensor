import type { Observable } from 'rxjs';
import type { Option } from '@polkadot/types';
import type { ReferendumInfoTo239 } from '@polkadot/types/interfaces';
import type { PalletDemocracyReferendumInfo } from '@polkadot/types/lookup';
import type { BN } from '@polkadot/util';
import type { DeriveApi, DeriveReferendum, DeriveReferendumVotes } from '../types.js';
export declare function _referendumVotes(instanceId: string, api: DeriveApi): (referendum: DeriveReferendum) => Observable<DeriveReferendumVotes>;
export declare function _referendumsVotes(instanceId: string, api: DeriveApi): (referendums: DeriveReferendum[]) => Observable<DeriveReferendumVotes[]>;
export declare function _referendumInfo(instanceId: string, api: DeriveApi): (index: BN, info: Option<PalletDemocracyReferendumInfo | ReferendumInfoTo239>) => Observable<DeriveReferendum | null>;
/**
 * @name referendumsInfo
 * @description Retrieves information about multiple referendums by their IDs.
 * @param {BN[]} ids An array of referendum IDs to query.
 * @example
 * ```javascript
 * import { BN } from "@polkadot/util";
 *
 * const referendumIds = [new BN(1)];
 * const referendums = await api.derive.democracy.referendumsInfo(referendumIds);
 * console.log("Referendums Info:", referendums);
 * ```
 */
export declare function referendumsInfo(instanceId: string, api: DeriveApi): (ids: BN[]) => Observable<DeriveReferendum[]>;
