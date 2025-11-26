import type { IsError } from '@polkadot/types/metadata/decorate/types';
import type { ApiTypes, EmptyBase } from './base.js';
export type AugmentedError<_ extends ApiTypes> = IsError;
export interface AugmentedErrors<ApiType extends ApiTypes> extends EmptyBase<ApiType> {
}
export interface DecoratedErrors<ApiType extends ApiTypes> extends AugmentedErrors<ApiType> {
    [key: string]: ModuleErrors<ApiType>;
}
export type ModuleErrors<ApiType extends ApiTypes> = Record<string, AugmentedError<ApiType>>;
