/**
 * @name initWasm
 * @description
 * For historic purposes and for tighter control on init, specifically performing
 * a WASM initialization with only ASM.js
 *
 * Generally should not be used unless you want explicit control over which
 * interfaces are initialized.
 */
export declare function initWasm(): Promise<void>;
