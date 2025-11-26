import type { Observable } from 'rxjs';
import type { AnyFunction, Codec, DefinitionCallNamed } from '@polkadot/types/types';
import type { ApiTypes, EmptyBase, ReturnCodec } from './base.js';
export type DecoratedCallBase<ApiType extends ApiTypes, F extends AnyFunction = (...args: any[]) => Observable<Codec>> = ApiType extends 'rxjs' ? <T = ReturnCodec<F>>(...args: Parameters<F>) => Observable<T> : <T = ReturnCodec<F>>(...args: Parameters<F>) => Promise<T>;
export type AugmentedCall<ApiType extends ApiTypes, F extends AnyFunction = (...args: any[]) => Observable<Codec>> = DecoratedCallBase<ApiType, F> & {
    /** The metadata/description/definition for this method */
    meta: DefinitionCallNamed;
};
export interface AugmentedCalls<ApiType extends ApiTypes> extends EmptyBase<ApiType> {
}
export interface QueryableCalls<ApiType extends ApiTypes> extends AugmentedCalls<ApiType> {
    [key: string]: QueryableModuleCalls<ApiType>;
}
export type QueryableModuleCalls<ApiType extends ApiTypes> = Record<string, DecoratedCallBase<ApiType>>;
