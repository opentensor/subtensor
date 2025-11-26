import type { Observable } from 'rxjs';
import type { BlockNumber } from '@polkadot/types/interfaces';
import type { DeriveApi, DeriveSessionProgress } from '../types.js';
/**
 * @name progress
 * @description Retrieves session information and progress.
 * @example
 * ```javascript
 * api.derive.session.progress((progress) => {
 *   console.log(`Session progress ${JSON.stringify(progress)}`);
 * });
 * ```
 */
export declare function progress(instanceId: string, api: DeriveApi): () => Observable<DeriveSessionProgress>;
/**
 * @name eraLenght
 * @description Retrieves the total length of the current era.
 * @example
 * ```javascript
 * api.derive.session.eraLength((length) => {
 *   console.log(`Current era length: ${length} sessions`);
 * });
 * ```
 */
export declare const eraLength: (instanceId: string, api: DeriveApi) => () => Observable<BlockNumber>;
/**
 * @name eraProgress
 * @description Retrieves the progress of the current era.
 * @example
 * ```javascript
 * api.derive.session.eraProgress((progress) => {
 *   console.log(`Current era progress: ${progress} sessions`);
 * });
 * ```
 */
export declare const eraProgress: (instanceId: string, api: DeriveApi) => () => Observable<BlockNumber>;
/**
 * @name sessionProgress
 * @description Retrieves the progress of the current session.
 * @example
 * ```javascript
 *   api.derive.session.sessionProgress((progress) => {
 *   console.log(`Current session progress: ${progress} slots`);
 * });
 * ```
 */
export declare const sessionProgress: (instanceId: string, api: DeriveApi) => () => Observable<BlockNumber>;
