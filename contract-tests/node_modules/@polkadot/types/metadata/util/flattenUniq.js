/** @internal */
export function flattenUniq(list, result = []) {
    for (let i = 0, count = list.length; i < count; i++) {
        const entry = list[i];
        if (Array.isArray(entry)) {
            flattenUniq(entry, result);
        }
        else {
            result.push(entry);
        }
    }
    return [...new Set(result)];
}
