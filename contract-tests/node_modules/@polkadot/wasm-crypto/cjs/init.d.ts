import type { InitFn } from '@polkadot/wasm-bridge/types';
import type { WasmCryptoInstance } from '@polkadot/wasm-crypto-init/types';
import { Bridge } from '@polkadot/wasm-bridge';
/**
 * @name bridge
 * @description
 * The JS <-> WASM bridge that is in operation. For the specific package
 * it is a global, i.e. all operations happens on this specific bridge
 */
export declare const bridge: Bridge<WasmCryptoInstance>;
/**
 * @name initBridge
 * @description
 * Creates a new bridge interface with the (optional) initialization function
 */
export declare function initBridge(createWasm?: InitFn<WasmCryptoInstance>): Promise<WasmCryptoInstance | null>;
