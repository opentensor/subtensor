"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.formatDate = formatDate;
/** @internal */
function zeroPad(value) {
    return value.toString().padStart(2, '0');
}
/**
 * @name formatDate
 * @description Formats a date in CCYY-MM-DD HH:MM:SS format
 */
function formatDate(date) {
    const year = date.getFullYear().toString();
    const month = zeroPad((date.getMonth() + 1));
    const day = zeroPad(date.getDate());
    const hour = zeroPad(date.getHours());
    const minute = zeroPad(date.getMinutes());
    const second = zeroPad(date.getSeconds());
    return `${year}-${month}-${day} ${hour}:${minute}:${second}`;
}
