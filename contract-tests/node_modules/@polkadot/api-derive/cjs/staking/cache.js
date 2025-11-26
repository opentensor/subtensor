"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getEraCache = getEraCache;
exports.getEraMultiCache = getEraMultiCache;
exports.setEraCache = setEraCache;
exports.setEraMultiCache = setEraMultiCache;
exports.filterCachedEras = filterCachedEras;
const index_js_1 = require("../util/index.js");
function getEraCache(CACHE_KEY, era, withActive) {
    const cacheKey = `${CACHE_KEY}-${era.toString()}`;
    return [
        cacheKey,
        withActive
            ? undefined
            : index_js_1.deriveCache.get(cacheKey)
    ];
}
function getEraMultiCache(CACHE_KEY, eras, withActive) {
    const cached = withActive
        ? []
        : eras
            .map((e) => index_js_1.deriveCache.get(`${CACHE_KEY}-${e.toString()}`))
            .filter((v) => !!v);
    return cached;
}
function setEraCache(cacheKey, withActive, value) {
    !withActive && index_js_1.deriveCache.set(cacheKey, value);
    return value;
}
function setEraMultiCache(CACHE_KEY, withActive, values) {
    !withActive && values.forEach((v) => index_js_1.deriveCache.set(`${CACHE_KEY}-${v.era.toString()}`, v));
    return values;
}
function filterCachedEras(eras, cached, query) {
    return eras
        .map((e) => cached.find(({ era }) => e.eq(era)) ||
        query.find(({ era }) => e.eq(era)))
        .filter((e) => !!e);
}
