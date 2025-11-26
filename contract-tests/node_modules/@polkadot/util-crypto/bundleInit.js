import '@polkadot/x-bigint/shim';
import { cryptoWaitReady } from './crypto.js';
cryptoWaitReady().catch(() => {
    // shouldn't happen, logged and caught inside cryptoWaitReady
});
