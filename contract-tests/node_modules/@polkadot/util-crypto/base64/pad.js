/**
 * @name base64Pad
 * @description Adds padding characters for correct length
 */
export function base64Pad(value) {
    return value.padEnd(value.length + (value.length % 4), '=');
}
