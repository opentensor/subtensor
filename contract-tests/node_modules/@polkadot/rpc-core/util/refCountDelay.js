import { asapScheduler, Observable, Subscription } from 'rxjs';
/** @internal */
export function refCountDelay(delay = 1750) {
    return (source) => {
        // state: 0 = disconnected, 1 = disconnecting, 2 = connecting, 3 = connected
        let [state, refCount, connection, scheduler] = [0, 0, Subscription.EMPTY, Subscription.EMPTY];
        return new Observable((ob) => {
            source.subscribe(ob);
            if (refCount++ === 0) {
                if (state === 1) {
                    scheduler.unsubscribe();
                }
                else {
                    // eslint-disable-next-line deprecation/deprecation
                    connection = source.connect();
                }
                state = 3;
            }
            return () => {
                if (--refCount === 0) {
                    if (state === 2) {
                        state = 0;
                        scheduler.unsubscribe();
                    }
                    else {
                        // state === 3
                        state = 1;
                        scheduler = asapScheduler.schedule(() => {
                            state = 0;
                            connection.unsubscribe();
                        }, delay);
                    }
                }
            };
        });
    };
}
