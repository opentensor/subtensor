import { CodecMap } from './Map.js';
export class HashMap extends CodecMap {
    static with(keyType, valType) {
        return class extends HashMap {
            constructor(registry, value) {
                super(registry, keyType, valType, value);
            }
        };
    }
}
