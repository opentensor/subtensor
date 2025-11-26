"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.objectSpread = objectSpread;
/**
 * @name objectSpread
 * @summary Concats all sources into the destination
 * @description Spreads object properties while maintaining object integrity
 */
function objectSpread(dest, ...sources) {
    const filterProps = new Set(['__proto__', 'constructor', 'prototype']);
    for (let i = 0, count = sources.length; i < count; i++) {
        const src = sources[i];
        if (src) {
            if (typeof src.entries === 'function') {
                for (const [key, value] of src.entries()) {
                    if (!filterProps.has(key)) {
                        dest[key] = value;
                    }
                }
            }
            else {
                // Create a clean copy of the source object
                const sanitizedSrc = Object.create(null);
                for (const [key, value] of Object.entries(src)) {
                    if (!filterProps.has(key)) {
                        sanitizedSrc[key] = value;
                    }
                }
                Object.assign(dest, sanitizedSrc);
            }
        }
    }
    return dest;
}
