/// <reference lib="dom" />
import { SmoldotBytecode } from './public-types.js';
/**
 * Compiles and returns the smoldot WebAssembly binary.
 */
export declare function compileBytecode(): Promise<SmoldotBytecode>;
