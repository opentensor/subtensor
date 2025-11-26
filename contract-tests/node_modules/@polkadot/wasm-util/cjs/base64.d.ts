/**
 * @name base64Decode
 * @description
 * A base64 decoding function that operates in all environments. Unlike decoding
 * from Buffer (Node.js only) or atob (browser-only) this implementation is
 * slightly slower, but it is platform independent.
 *
 * For our usage, since we have access to the static final size, so we decode
 * to a specified output buffer. This also means we have applied a number of
 * optimizations based on this - checking output position instead of chars.
 */
export declare function base64Decode(data: string, out: Uint8Array): Uint8Array;
