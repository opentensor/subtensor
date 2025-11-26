import type { DeriveCustom } from '@polkadot/api-base/types';
import type { ExactDerive } from './derive.js';
import type { DeriveApi } from './types.js';
import { lazyDeriveSection } from './util/index.js';
export * from './derive.js';
export * from './type/index.js';
export { lazyDeriveSection };
/** @internal */
export declare function getAvailableDerives(instanceId: string, api: DeriveApi, custom?: DeriveCustom): ExactDerive;
