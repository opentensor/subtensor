import { nobody } from './pair/nobody.js';
import { createTestKeyring } from './testing.js';
export function createTestPairs(options, isDerived = true) {
    const keyring = createTestKeyring(options, isDerived);
    const pairs = keyring.getPairs();
    const map = { nobody: nobody() };
    for (const p of pairs) {
        if (p.meta.name) {
            map[p.meta.name] = p;
        }
    }
    return map;
}
