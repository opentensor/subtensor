import { isString, isU8a, u8aToU8a } from '@polkadot/util';
import { MetadataVersioned } from './MetadataVersioned.js';
const EMPTY_METADATA = new Uint8Array([0x6d, 0x65, 0x74, 0x61, 9]);
const VERSION_IDX = EMPTY_METADATA.length - 1;
/** @internal */
function decodeU8a(registry, u8a) {
    if (u8a.length === 0) {
        return EMPTY_METADATA;
    }
    else if (u8a[VERSION_IDX] === 9) {
        // This is an f-ing hack as a follow-up to another ugly hack
        // https://github.com/polkadot-js/api/commit/a9211690be6b68ad6c6dad7852f1665cadcfa5b2
        // when we fail on V9, try to re-parse it as v10...
        try {
            return new MetadataVersioned(registry, u8a);
        }
        catch {
            u8a[VERSION_IDX] = 10;
        }
    }
    return u8a;
}
/**
 * @name Metadata
 * @description
 * The versioned runtime metadata as a decoded structure
 */
export class Metadata extends MetadataVersioned {
    constructor(registry, value) {
        // const timeStart = performance.now()
        super(registry, isU8a(value) || isString(value)
            ? decodeU8a(registry, u8aToU8a(value))
            : value);
        // console.log('Metadata', `${(performance.now() - timeStart).toFixed(2)}ms`)
    }
}
