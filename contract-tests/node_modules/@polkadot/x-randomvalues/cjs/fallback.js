"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.insecureRandomValues = insecureRandomValues;
let warned = false;
function insecureRandomValues(arr) {
    if (!warned) {
        console.warn('Using an insecure random number generator, this should only happen when running in a debugger without support for crypto');
        warned = true;
    }
    let r = 0;
    for (let i = 0, count = arr.length; i < count; i++) {
        if ((i & 0b11) === 0) {
            r = Math.random() * 0x100000000;
        }
        arr[i] = (r >>> ((i & 0b11) << 3)) & 0xff;
    }
    return arr;
}
