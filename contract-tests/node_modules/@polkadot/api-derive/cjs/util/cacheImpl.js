"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.deriveNoopCache = exports.deriveMapCache = void 0;
const mapCache = new Map();
exports.deriveMapCache = {
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
exports.deriveNoopCache = {
    del: () => undefined,
    forEach: () => undefined,
    get: () => undefined,
    set: (_, value) => value
};
