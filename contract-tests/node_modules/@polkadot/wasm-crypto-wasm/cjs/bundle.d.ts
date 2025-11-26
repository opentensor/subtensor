export { packageInfo } from './packageInfo.js';
/**
 * @name wasmBytes
 * @description
 * The decoded WASM interface as exposed by this package.
 *
 * The build process will output into cjs/* into a compressed base64 format.
 * Upon loading the exposed bytes will be decoded and decompressed from this
 * specific format and returned.
 */
export declare const wasmBytes: Uint8Array;
