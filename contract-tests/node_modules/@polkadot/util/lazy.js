/**
 * @name lazyMethod
 * @description
 * Creates a lazy, on-demand getter for the specific value. Upon get the value will be evaluated.
 */
export function lazyMethod(result, item, creator, getName, index = 0) {
    const name = getName
        ? getName(item, index)
        : item.toString();
    let value;
    Object.defineProperty(result, name, {
        // This allows for re-configuration with the embedded defineProperty below
        // and ensures that on tested browsers and Node, it _will_ be redefined
        // and thus short-circuited for future access
        configurable: true,
        enumerable: true,
        // Use a function here, we don't want to capture the outer this, i.e.
        // don't use arrow functions in this context since we have a this inside
        get: function () {
            // This check should _always_ be false and unneeded, since we override
            // with a value below ... however we ensure we are quire vigilant against
            // all environment failures, so we are rather be safe than sorry
            if (value === undefined) {
                value = creator(item, index, this);
                try {
                    // re-define the property as a value, next time around this
                    // getter will only return the computed value
                    Object.defineProperty(this, name, { value });
                }
                catch {
                    // ignore any errors, since this _should_ not happen due to
                    // the "configurable" property above. But if it ever does
                    // from here-on we will be the cached value the next time
                    // around (with a very slight dip in performance)
                }
            }
            return value;
        }
    });
}
/**
 * @name lazyMethods
 * @description
 * Creates lazy, on-demand getters for the specific values.
 */
export function lazyMethods(result, items, creator, getName) {
    for (let i = 0, count = items.length; i < count; i++) {
        lazyMethod(result, items[i], creator, getName, i);
    }
    return result;
}
