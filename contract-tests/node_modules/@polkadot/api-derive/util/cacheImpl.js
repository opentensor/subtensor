const mapCache = new Map();
export const deriveMapCache = {
    del: (key) => {
        mapCache.delete(key);
    },
    forEach: (cb) => {
        for (const [k, v] of mapCache.entries()) {
            cb(k, v);
        }
    },
    get: (key) => {
        return mapCache.get(key);
    },
    set: (key, value) => {
        mapCache.set(key, value);
    }
};
export const deriveNoopCache = {
    del: () => undefined,
    forEach: () => undefined,
    get: () => undefined,
    set: (_, value) => value
};
