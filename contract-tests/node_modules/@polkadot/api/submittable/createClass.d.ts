import type { CodecClass } from '@polkadot/types/types';
import type { ApiBase } from '../base/index.js';
import type { ApiInterfaceRx, ApiTypes } from '../types/index.js';
import type { SubmittableExtrinsic } from './types.js';
interface SubmittableOptions<ApiType extends ApiTypes> {
    api: ApiInterfaceRx;
    apiType: ApiTypes;
    blockHash?: Uint8Array | undefined;
    decorateMethod: ApiBase<ApiType>['_decorateMethod'];
}
export declare function createClass<ApiType extends ApiTypes>({ api, apiType, blockHash, decorateMethod }: SubmittableOptions<ApiType>): CodecClass<SubmittableExtrinsic<ApiType>>;
export {};
