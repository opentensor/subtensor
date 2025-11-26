"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.refCountDelay = refCountDelay;
const rxjs_1 = require("rxjs");
/** @internal */
function refCountDelay(delay = 1750) {
    return (source) => {
        // state: 0 = disconnected, 1 = disconnecting, 2 = connecting, 3 = connected
        let [state, refCount, connection, scheduler] = [0, 0, rxjs_1.Subscription.EMPTY, rxjs_1.Subscription.EMPTY];
        return new rxjs_1.Observable((ob) => {
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
                        scheduler = rxjs_1.asapScheduler.schedule(() => {
                            state = 0;
                            connection.unsubscribe();
                        }, delay);
                    }
                }
            };
        });
    };
}
