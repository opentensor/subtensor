/**
 * @internal
 * From a type string or class, return the associated type class
 */
export function typeToConstructor(registry, type) {
    return typeof type === 'function'
        ? type
        : registry.createClassUnsafe(type);
}
/**
 * @internal
 * Takes an input array of types and returns the associated classes for it
*/
export function typesToConstructors(registry, types) {
    const count = types.length;
    const result = new Array(count);
    for (let i = 0; i < count; i++) {
        result[i] = typeToConstructor(registry, types[i]);
    }
    return result;
}
/**
 * @internal
 * Takes an input map of the form `{ [string]: string | CodecClass }` and returns a map of `{ [string]: CodecClass }`
 */
export function mapToTypeMap(registry, input) {
    const entries = Object.entries(input);
    const count = entries.length;
    const output = [new Array(count), new Array(count)];
    for (let i = 0; i < count; i++) {
        output[1][i] = entries[i][0];
        output[0][i] = typeToConstructor(registry, entries[i][1]);
    }
    return output;
}
