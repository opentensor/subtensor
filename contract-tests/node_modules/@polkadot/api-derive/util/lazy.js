import { lazyMethod, lazyMethods } from '@polkadot/util';
export function lazyDeriveSection(result, section, getKeys, creator) {
    lazyMethod(result, section, () => lazyMethods({}, getKeys(section), (method) => creator(section, method)));
}
