import { createWasm } from '@polkadot/wasm-crypto-init/wasm';
import { initBridge } from './init.js';
/**
 * @name initWasm
 * @description
 * For historic purposes and for tighter control on init, specifically performing
 * a WASM initialization with only WASM (generally the default for most platforms)
 *
 * Generally should not be used unless you want explicit control over which
 * interfaces are initialized.
 */
export async function initWasm() {
    await initBridge(createWasm);
}
initWasm().catch(() => {
    // cannot happen, initWasm doesn't throw
});
