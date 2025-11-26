import type { Memoized } from '@polkadot/util/types';
import { Observable } from 'rxjs';
type ObsFn<T> = (...params: unknown[]) => Observable<T>;
/** @internal */
export declare function memo<T>(instanceId: string, inner: Function): Memoized<ObsFn<T>>;
export {};
