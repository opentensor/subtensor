import { isFunction, noop } from '@polkadot/util';
export class Combinator {
    #allHasFired = false;
    #callback;
    #fired = [];
    #fns = [];
    #isActive = true;
    #results = [];
    #subscriptions = [];
    constructor(fns, callback) {
        this.#callback = callback;
        // eslint-disable-next-line @typescript-eslint/no-floating-promises, @typescript-eslint/require-await
        this.#subscriptions = fns.map(async (input, index) => {
            const [fn, ...args] = Array.isArray(input)
                ? input
                : [input];
            this.#fired.push(false);
            this.#fns.push(fn);
            // Not quite 100% how to have a variable number at the front here
            // eslint-disable-next-line @typescript-eslint/no-unsafe-return,@typescript-eslint/ban-types
            return fn(...args, this._createCallback(index));
        });
    }
    _allHasFired() {
        this.#allHasFired ||= this.#fired.filter((hasFired) => !hasFired).length === 0;
        return this.#allHasFired;
    }
    _createCallback(index) {
        return (value) => {
            this.#fired[index] = true;
            this.#results[index] = value;
            this._triggerUpdate();
        };
    }
    _triggerUpdate() {
        if (!this.#isActive || !isFunction(this.#callback) || !this._allHasFired()) {
            return;
        }
        try {
            Promise
                .resolve(this.#callback(this.#results))
                .catch(noop);
        }
        catch {
            // swallow, we don't want the handler to trip us up
        }
    }
    unsubscribe() {
        if (!this.#isActive) {
            return;
        }
        this.#isActive = false;
        Promise
            .all(this.#subscriptions.map(async (subscription) => {
            try {
                const unsubscribe = await subscription;
                if (isFunction(unsubscribe)) {
                    unsubscribe();
                }
            }
            catch {
                // ignore
            }
        })).catch(() => {
            // ignore, already ignored above, should never throw
        });
    }
}
