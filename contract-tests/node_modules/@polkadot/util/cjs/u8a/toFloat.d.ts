interface Options {
    bitLength?: 32 | 64;
    isLe?: boolean;
}
/**
 * @name u8aToFloat
 * @description Converts a Uint8Array value into the float (either 32 or 64-bit)
 * representation.
 */
export declare function u8aToFloat(value: Uint8Array, { bitLength, isLe }?: Options): number;
export {};
