"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hasProcess = exports.hasBuffer = exports.hasWasm = exports.hasEsm = exports.hasDirname = exports.hasCjs = exports.hasBigInt = void 0;
const x_bigint_1 = require("@polkadot/x-bigint");
const x_global_1 = require("@polkadot/x-global");
/** true if the environment has proper BigInt support */
exports.hasBigInt = typeof x_bigint_1.BigInt === 'function' && typeof x_bigint_1.BigInt.asIntN === 'function';
/** true if the environment is CJS */
exports.hasCjs = typeof require === 'function' && typeof module !== 'undefined';
/** true if the environment has __dirname available */
exports.hasDirname = typeof __dirname !== 'undefined';
/** true if the environment is ESM */
exports.hasEsm = !exports.hasCjs;
/** true if the environment has WebAssembly available */
exports.hasWasm = typeof WebAssembly !== 'undefined';
/** true if the environment has support for Buffer (typically Node.js) */
exports.hasBuffer = typeof x_global_1.xglobal.Buffer === 'function' && typeof x_global_1.xglobal.Buffer.isBuffer === 'function';
/** true if the environment has process available (typically Node.js) */
exports.hasProcess = typeof x_global_1.xglobal.process === 'object';
