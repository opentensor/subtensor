import { catchError, distinctUntilChanged, publishReplay, refCount, tap } from 'rxjs';
import { stringify } from '@polkadot/util';
import { refCountDelay } from './refCountDelay.js';
function CMP(a, b) {
    return stringify({ t: a }) === stringify({ t: b });
}
function ERR(error) {
    throw error;
}
function NOOP() {
    // empty
}
/**
 * Shorthand for distinctUntilChanged(), publishReplay(1) and refCount().
 *
 * @ignore
 * @internal
 */
export function drr({ delay, skipChange = false, skipTimeout = false } = {}) {
    return (source$) => source$.pipe(catchError(ERR), skipChange
        ? tap(NOOP)
        : distinctUntilChanged(CMP), 
    // eslint-disable-next-line deprecation/deprecation
    publishReplay(1), skipTimeout
        // eslint-disable-next-line deprecation/deprecation
        ? refCount()
        : refCountDelay(delay));
}
