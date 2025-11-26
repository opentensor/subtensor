import { createWasmFn } from '@polkadot/wasm-bridge';
import { wasmBytes } from '@polkadot/wasm-crypto-wasm';
export { packageInfo } from './packageInfo.js';
/**
 * @name createWasm
 * @description
 * Creates an interface using only WASM
 */
export const createWasm = /*#__PURE__*/ createWasmFn('crypto', wasmBytes, null);
