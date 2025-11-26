"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createWasmFn = createWasmFn;
/**
 * @name createWasmFn
 * @description
 * Create a WASM (or ASM.js) creator interface based on the supplied information.
 *
 * It will attempt to create a WASM interface first and if this fails or is not available in
 * the environment, will fallback to attempting to create an ASM.js interface.
 */
function createWasmFn(root, wasmBytes, asmFn) {
    return async (wbg) => {
        const result = {
            error: null,
            type: 'none',
            wasm: null
        };
        try {
            if (!wasmBytes?.length) {
                throw new Error('No WebAssembly provided for initialization');
            }
            else if (typeof WebAssembly !== 'object' || typeof WebAssembly.instantiate !== 'function') {
                throw new Error('WebAssembly is not available in your environment');
            }
            const source = await WebAssembly.instantiate(wasmBytes, { wbg });
            result.wasm = source.instance.exports;
            result.type = 'wasm';
        }
        catch (error) {
            // if we have a valid supplied asm.js, return that
            if (typeof asmFn === 'function') {
                result.wasm = asmFn(wbg);
                result.type = 'asm';
            }
            else {
                result.error = `FATAL: Unable to initialize @polkadot/wasm-${root}:: ${error.message}`;
                console.error(result.error);
            }
        }
        return result;
    };
}
