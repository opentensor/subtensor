"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.nextTick = nextTick;
/**
 * @name nextTick
 * @description Defer the operation to the queue for evaluation on the next tick
 */
function nextTick(onExec, onError) {
    // While Promise.resolve().then(...) would defer to the nextTick, this
    // actually does not play as nicely in browsers like the setTimeout(...)
    // approach. So the safer, though less optimal approach is the one taken here
    setTimeout(() => {
        Promise
            .resolve()
            .then(() => {
            onExec();
        })
            .catch((error) => {
            if (onError) {
                onError(error);
            }
            else {
                console.error(error);
            }
        });
    }, 0);
}
