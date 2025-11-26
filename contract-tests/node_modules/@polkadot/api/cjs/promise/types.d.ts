import type { SubmittableExtrinsic as SubmittableExtrinsicBase } from '../submittable/types.js';
import type { QueryableStorageEntry as QueryableStorageEntryBase, SubmittableExtrinsicFunction as SubmittableExtrinsicFunctionBase } from '../types/index.js';
export type QueryableStorageEntry = QueryableStorageEntryBase<'promise'>;
export type SubmittableExtrinsic = SubmittableExtrinsicBase<'promise'>;
export type SubmittableExtrinsicFunction = SubmittableExtrinsicFunctionBase<'promise'>;
