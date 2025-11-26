"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.MagicNumber = exports.MAGIC_NUMBER = void 0;
const types_codec_1 = require("@polkadot/types-codec");
exports.MAGIC_NUMBER = 0x6174656d; // `meta`, reversed for Little Endian encoding
class MagicNumber extends types_codec_1.U32 {
    constructor(registry, value) {
        super(registry, value);
        if (!this.isEmpty && !this.eq(exports.MAGIC_NUMBER)) {
            throw new Error(`MagicNumber mismatch: expected ${registry.createTypeUnsafe('u32', [exports.MAGIC_NUMBER]).toHex()}, found ${this.toHex()}`);
        }
    }
}
exports.MagicNumber = MagicNumber;
