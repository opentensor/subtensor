import type { Metadata } from '@polkadot/types';
import type { RuntimeVersionPartial } from '@polkadot/types/interfaces';
import type { DecoratedMeta } from '@polkadot/types/metadata/decorate/types';
import type { Registry } from '@polkadot/types/types';
import type { ApiDecoration, ApiTypes } from '../types/index.js';
export interface VersionedRegistry<ApiType extends ApiTypes> {
    counter: number;
    decoratedApi?: ApiDecoration<ApiType>;
    decoratedMeta?: DecoratedMeta;
    isDefault?: boolean;
    lastBlockHash?: Uint8Array | null;
    metadata: Metadata;
    registry: Registry;
    runtimeVersion: RuntimeVersionPartial;
}
