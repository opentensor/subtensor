import { isReady, waitReady } from '@polkadot/wasm-crypto';
export const cryptoIsReady = isReady;
export function cryptoWaitReady() {
    return waitReady()
        .then(() => {
        if (!isReady()) {
            throw new Error('Unable to initialize @polkadot/util-crypto');
        }
        return true;
    })
        .catch(() => false);
}
