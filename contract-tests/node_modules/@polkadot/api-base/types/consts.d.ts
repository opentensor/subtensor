import type { PalletConstantMetadataLatest } from '@polkadot/types/interfaces';
import type { Codec } from '@polkadot/types/types';
import type { ApiTypes, EmptyBase } from './base.js';
export interface AugmentedConst<_ extends ApiTypes> {
    meta: PalletConstantMetadataLatest;
}
export interface AugmentedConsts<ApiType extends ApiTypes> extends EmptyBase<ApiType> {
}
export interface QueryableConsts<ApiType extends ApiTypes> extends AugmentedConsts<ApiType> {
    [key: string]: QueryableModuleConsts;
}
export type QueryableModuleConsts = Record<string, Codec>;
