"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.augmentObject = augmentObject;
const util_1 = require("@polkadot/util");
const l = (0, util_1.logger)('api/augment');
function logLength(type, values, and = []) {
    return values.length
        ? ` ${values.length} ${type}${and.length ? ' and' : ''}`
        : '';
}
function logValues(type, values) {
    return values.length
        ? `\n\t${type.padStart(7)}: ${values.sort().join(', ')}`
        : '';
}
function warn(prefix, type, [added, removed]) {
    if (added.length || removed.length) {
        l.warn(`api.${prefix}: Found${logLength('added', added, removed)}${logLength('removed', removed)} ${type}:${logValues('added', added)}${logValues('removed', removed)}`);
    }
}
function findSectionExcludes(a, b) {
    return a.filter((s) => !b.includes(s));
}
function findSectionIncludes(a, b) {
    return a.filter((s) => b.includes(s));
}
function extractSections(src, dst) {
    const srcSections = Object.keys(src);
    const dstSections = Object.keys(dst);
    return [
        findSectionExcludes(srcSections, dstSections),
        findSectionExcludes(dstSections, srcSections)
    ];
}
function findMethodExcludes(src, dst) {
    const srcSections = Object.keys(src);
    const dstSections = findSectionIncludes(Object.keys(dst), srcSections);
    const excludes = [];
    for (let s = 0, scount = dstSections.length; s < scount; s++) {
        const section = dstSections[s];
        const srcMethods = Object.keys(src[section]);
        const dstMethods = Object.keys(dst[section]);
        for (let d = 0, mcount = dstMethods.length; d < mcount; d++) {
            const method = dstMethods[d];
            if (!srcMethods.includes(method)) {
                excludes.push(`${section}.${method}`);
            }
        }
    }
    return excludes;
}
function extractMethods(src, dst) {
    return [
        findMethodExcludes(dst, src),
        findMethodExcludes(src, dst)
    ];
}
/**
 * @description Takes a decorated api section (e.g. api.tx) and augment it with the details. It does not override what is
 * already available, but rather just adds new missing items into the result object.
 * @internal
 */
function augmentObject(prefix, src, dst, fromEmpty = false) {
    fromEmpty && (0, util_1.objectClear)(dst);
    // NOTE: This part is slightly problematic since it will get the
    // values for at least all the sections and the names of the methods
    // (Since methods won't be decorated before lazy, this _may_ be ok)
    if (prefix && Object.keys(dst).length) {
        warn(prefix, 'modules', extractSections(src, dst));
        warn(prefix, 'calls', extractMethods(src, dst));
    }
    const sections = Object.keys(src);
    for (let i = 0, count = sections.length; i < count; i++) {
        const section = sections[i];
        const methods = src[section];
        // We don't set here with a lazy interface, we decorate based
        // on the top-level structure (this bypasses adding lazy onto lazy)
        if (!dst[section]) {
            dst[section] = {};
        }
        (0, util_1.lazyMethods)(dst[section], Object.keys(methods), (m) => methods[m]);
    }
    return dst;
}
