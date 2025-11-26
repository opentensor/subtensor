import { U32 } from '@polkadot/types-codec';
export const MAGIC_NUMBER = 0x6174656d; // `meta`, reversed for Little Endian encoding
export class MagicNumber extends U32 {
    constructor(registry, value) {
        super(registry, value);
        if (!this.isEmpty && !this.eq(MAGIC_NUMBER)) {
            throw new Error(`MagicNumber mismatch: expected ${registry.createTypeUnsafe('u32', [MAGIC_NUMBER]).toHex()}, found ${this.toHex()}`);
        }
    }
}
