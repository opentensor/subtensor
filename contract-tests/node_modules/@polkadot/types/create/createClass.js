import { createClassUnsafe } from '@polkadot/types-create';
export function createClass(registry, type) {
    return createClassUnsafe(registry, type);
}
