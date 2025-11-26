import type { Registry } from '@polkadot/types-codec/types';
import type { MetaVersionAll } from '../versions.js';
import type { Check } from './types.js';
/** @internal */
export declare function decodeLatestMeta(registry: Registry, type: string, version: MetaVersionAll, { data }: Check): void;
/** @internal */
export declare function toLatest(registry: Registry, version: MetaVersionAll, { data }: Check, withThrow?: boolean): void;
/** @internal */
export declare function defaultValues(registry: Registry, { data, fails }: Check, withThrow: boolean | undefined, withFallbackCheck: boolean | undefined, version: MetaVersionAll): void;
export declare function testMeta(version: MetaVersionAll, matchers: Record<string, Check>, withFallback?: boolean): void;
