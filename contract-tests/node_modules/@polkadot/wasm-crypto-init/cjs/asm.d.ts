import type { InitFn } from '@polkadot/wasm-bridge/types';
import type { WasmCryptoInstance } from './types.js';
export { packageInfo } from './packageInfo.js';
/**
 * @name createWasm
 * @description
 * Creates an interface using only ASM.js
 */
export declare const createWasm: InitFn<WasmCryptoInstance>;
