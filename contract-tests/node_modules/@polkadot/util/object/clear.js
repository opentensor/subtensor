/**
 * @name objectClear
 * @summary Removes all the keys from the input object
 */
export function objectClear(value) {
    const keys = Object.keys(value);
    for (let i = 0, count = keys.length; i < count; i++) {
        delete value[keys[i]];
    }
    return value;
}
