import type { IsEvent } from '@polkadot/types/metadata/decorate/types';
import type { AnyTuple } from '@polkadot/types/types';
import type { ApiTypes, EmptyBase } from './base.js';
export type AugmentedEvent<_ extends ApiTypes, T extends AnyTuple = AnyTuple, N = unknown> = IsEvent<T, N>;
export interface AugmentedEvents<ApiType extends ApiTypes> extends EmptyBase<ApiType> {
}
export interface DecoratedEvents<ApiType extends ApiTypes> extends AugmentedEvents<ApiType> {
    [key: string]: ModuleEvents<ApiType>;
}
export type ModuleEvents<ApiType extends ApiTypes> = Record<string, AugmentedEvent<ApiType>>;
