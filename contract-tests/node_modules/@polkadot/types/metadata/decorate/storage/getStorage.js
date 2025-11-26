import { substrate } from './substrate.js';
/** @internal */
export function getStorage(registry) {
    const storage = {};
    const entries = Object.entries(substrate);
    for (let e = 0, count = entries.length; e < count; e++) {
        storage[entries[e][0]] = entries[e][1](registry);
    }
    return { substrate: storage };
}
