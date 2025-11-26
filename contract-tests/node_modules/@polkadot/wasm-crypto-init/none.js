import { createWasmFn } from '@polkadot/wasm-bridge';
export { packageInfo } from './packageInfo.js';
/**
 * @name createWasm
 * @description
 * Creates an interface using no WASM and no ASM.js
 */
export const createWasm = /*#__PURE__*/ createWasmFn('crypto', null, null);
