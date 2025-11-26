import type { Call } from '@polkadot/types/interfaces';
import type { Registry } from '@polkadot/types-codec/types';
import type { ApiBase } from '../base/index.js';
import type { ApiInterfaceRx, ApiTypes } from '../types/index.js';
import type { SubmittableExtrinsic } from './types.js';
type Creator<ApiType extends ApiTypes> = (extrinsic: Call | Uint8Array | string) => SubmittableExtrinsic<ApiType>;
export declare function createSubmittable<ApiType extends ApiTypes>(apiType: ApiTypes, api: ApiInterfaceRx, decorateMethod: ApiBase<ApiType>['_decorateMethod'], registry?: Registry, blockHash?: Uint8Array): Creator<ApiType>;
export {};
