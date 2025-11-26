import { BehaviorSubject, combineLatest, map, of, switchMap, tap, toArray } from 'rxjs';
import { arrayChunk, arrayFlatten, nextTick } from '@polkadot/util';
import { memo } from '../util/index.js';
const ERA_CHUNK_SIZE = 14;
function chunkEras(eras, fn) {
    const chunked = arrayChunk(eras, ERA_CHUNK_SIZE);
    let index = 0;
    const subject = new BehaviorSubject(chunked[index]);
    return subject.pipe(switchMap(fn), tap(() => {
        nextTick(() => {
            index++;
            index === chunked.length
                ? subject.complete()
                : subject.next(chunked[index]);
        });
    }), toArray(), map(arrayFlatten));
}
export function filterEras(eras, list) {
    return eras.filter((e) => !list.some(({ era }) => e.eq(era)));
}
export function erasHistoricApply(fn) {
    return (instanceId, api) => 
    // Cannot quite get the typing right, but it is right in the code
    // eslint-disable-next-line @typescript-eslint/no-unsafe-return
    memo(instanceId, (withActive = false) => api.derive.staking.erasHistoric(withActive).pipe(switchMap((e) => api.derive.staking[fn](e, withActive))));
}
export function erasHistoricApplyAccount(fn) {
    return (instanceId, api) => 
    // Cannot quite get the typing right, but it is right in the code
    // eslint-disable-next-line @typescript-eslint/no-unsafe-return
    memo(instanceId, (accountId, withActive = false, page) => api.derive.staking.erasHistoric(withActive).pipe(switchMap((e) => api.derive.staking[fn](accountId, e, withActive, page || 0))));
}
export function singleEra(fn) {
    return (instanceId, api) => 
    // Cannot quite get the typing right, but it is right in the code
    // eslint-disable-next-line @typescript-eslint/no-unsafe-return
    memo(instanceId, (era) => api.derive.staking[fn](era, true));
}
export function combineEras(fn) {
    return (instanceId, api) => 
    // Cannot quite get the typing right, but it is right in the code
    // eslint-disable-next-line @typescript-eslint/no-unsafe-return
    memo(instanceId, (eras, withActive) => !eras.length
        ? of([])
        : chunkEras(eras, (eras) => combineLatest(eras.map((e) => api.derive.staking[fn](e, withActive)))));
}
