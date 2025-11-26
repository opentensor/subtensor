"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.identity = identity;
exports.noop = noop;
/**
 * A sharable identity function. Returns the input as-is with no transformation applied.
 */
function identity(value) {
    return value;
}
/**
 * A sharable noop function. As the name suggests, does nothing
 */
function noop() {
    // noop
}
