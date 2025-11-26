"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.base64Pad = base64Pad;
/**
 * @name base64Pad
 * @description Adds padding characters for correct length
 */
function base64Pad(value) {
    return value.padEnd(value.length + (value.length % 4), '=');
}
