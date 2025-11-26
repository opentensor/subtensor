import { CodecMap } from './Map.js';
export class BTreeMap extends CodecMap {
    static with(keyType, valType) {
        return class extends BTreeMap {
            constructor(registry, value) {
                super(registry, keyType, valType, value, 'BTreeMap');
            }
        };
    }
}
