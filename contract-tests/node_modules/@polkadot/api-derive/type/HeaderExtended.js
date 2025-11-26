import { extractAuthor } from './util.js';
export function createHeaderExtended(registry, header, validators, author) {
    // an instance of the base extrinsic for us to extend
    const HeaderBase = registry.createClass('Header');
    class Implementation extends HeaderBase {
        #author;
        constructor(registry, header, validators, author) {
            super(registry, header);
            this.#author = author || extractAuthor(this.digest, validators || []);
            this.createdAtHash = header?.createdAtHash;
        }
        /**
         * @description Convenience method, returns the author for the block
         */
        get author() {
            return this.#author;
        }
    }
    return new Implementation(registry, header, validators, author);
}
