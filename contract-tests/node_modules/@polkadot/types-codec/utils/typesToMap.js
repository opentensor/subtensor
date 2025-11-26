export function typesToMap(registry, [Types, keys]) {
    const result = {};
    for (let i = 0, count = keys.length; i < count; i++) {
        result[keys[i]] = registry.getClassName(Types[i]) || new Types[i](registry).toRawType();
    }
    return result;
}
