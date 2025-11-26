import { createWasmFn } from '@polkadot/wasm-bridge';
import * as mod from '@polkadot/wasm-crypto-asmjs';
const { asmJsInit } = mod;
export { packageInfo } from './packageInfo.js';
/**
 * @name createWasm
 * @description
 * Creates an interface using only ASM.js
 */
export const createWasm = /*#__PURE__*/ createWasmFn('crypto', null, asmJsInit);
