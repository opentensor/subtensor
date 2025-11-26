import { BigInt } from '@polkadot/x-bigint';
import { xglobal } from '@polkadot/x-global';
/** true if the environment has proper BigInt support */
export const hasBigInt = typeof BigInt === 'function' && typeof BigInt.asIntN === 'function';
/** true if the environment is CJS */
export const hasCjs = typeof require === 'function' && typeof module !== 'undefined';
/** true if the environment has __dirname available */
export const hasDirname = typeof __dirname !== 'undefined';
/** true if the environment is ESM */
export const hasEsm = !hasCjs;
/** true if the environment has WebAssembly available */
export const hasWasm = typeof WebAssembly !== 'undefined';
/** true if the environment has support for Buffer (typically Node.js) */
export const hasBuffer = typeof xglobal.Buffer === 'function' && typeof xglobal.Buffer.isBuffer === 'function';
/** true if the environment has process available (typically Node.js) */
export const hasProcess = typeof xglobal.process === 'object';
