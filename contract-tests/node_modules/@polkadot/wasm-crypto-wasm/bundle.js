import { base64Decode, unzlibSync } from '@polkadot/wasm-util';
import { bytes, lenIn, lenOut } from './cjs/bytes.js';
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
export const wasmBytes = /*#__PURE__*/ unzlibSync(base64Decode(bytes, new Uint8Array(lenIn)), new Uint8Array(lenOut));
