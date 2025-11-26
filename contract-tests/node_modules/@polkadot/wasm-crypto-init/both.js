import { createWasmFn } from '@polkadot/wasm-bridge';
import { asmJsInit } from '@polkadot/wasm-crypto-asmjs';
import { wasmBytes } from '@polkadot/wasm-crypto-wasm';
export { packageInfo } from './packageInfo.js';
/**
 * @name createWasm
 * @description
 * Creates an interface using WASM and a fallback ASM.js
 */
export const createWasm = /*#__PURE__*/ createWasmFn('crypto', wasmBytes, asmJsInit);
