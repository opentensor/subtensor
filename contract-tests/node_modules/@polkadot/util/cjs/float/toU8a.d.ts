interface Options {
    bitLength?: 32 | 64;
    isLe?: boolean;
}
/**
 * @name floatToU8a
 * @description Converts a float into a U8a representation (While we don't use BE in SCALE
 * we still allow for either representation, although, as elsewhere, isLe is default)
 */
export declare function floatToU8a(value?: String | string | number | Number, { bitLength, isLe }?: Options): Uint8Array;
export {};
