import { lazyDeriveSection } from '@polkadot/api-derive';
/**
 * This is a section decorator which keeps all type information.
 */
export function decorateDeriveSections(decorateMethod, derives) {
    const getKeys = (s) => Object.keys(derives[s]);
    const creator = (s, m) => decorateMethod(derives[s][m]);
    const result = {};
    const names = Object.keys(derives);
    for (let i = 0, count = names.length; i < count; i++) {
        lazyDeriveSection(result, names[i], getKeys, creator);
    }
    return result;
}
