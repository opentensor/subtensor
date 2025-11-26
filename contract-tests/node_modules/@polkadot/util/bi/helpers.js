/** @internal */
export function createCmp(cmp) {
    return (...items) => {
        const count = items.length;
        if (count === 0) {
            throw new Error('Must provide one or more arguments');
        }
        let result = items[0];
        for (let i = 1; i < count; i++) {
            if (cmp(items[i], result)) {
                result = items[i];
            }
        }
        return result;
    };
}
