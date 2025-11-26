import { Bridge } from '@polkadot/wasm-bridge';
import { createWasm } from '@polkadot/wasm-crypto-init';
/**
 * @name bridge
 * @description
 * The JS <-> WASM bridge that is in operation. For the specific package
 * it is a global, i.e. all operations happens on this specific bridge
 */
export const bridge = new Bridge(createWasm);
/**
 * @name initBridge
 * @description
 * Creates a new bridge interface with the (optional) initialization function
 */
export async function initBridge(createWasm) {
    return bridge.init(createWasm);
}
