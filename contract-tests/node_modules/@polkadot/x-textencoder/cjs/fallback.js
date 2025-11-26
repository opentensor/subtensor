"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TextEncoder = void 0;
class TextEncoder {
    encode(value) {
        const count = value.length;
        const u8a = new Uint8Array(count);
        for (let i = 0; i < count; i++) {
            u8a[i] = value.charCodeAt(i);
        }
        return u8a;
    }
}
exports.TextEncoder = TextEncoder;
