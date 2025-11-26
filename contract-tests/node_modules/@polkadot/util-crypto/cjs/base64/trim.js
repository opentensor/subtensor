"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.base64Trim = base64Trim;
/**
 * @name base64Trim
 * @description Trims padding characters
 */
function base64Trim(value) {
    while (value.length && value.endsWith('=')) {
        value = value.slice(0, -1);
    }
    return value;
}
