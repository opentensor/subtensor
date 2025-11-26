import { Struct } from '@polkadot/types-codec';
import { getUniqTypes, toCallsOnly } from './util/index.js';
import { toV10 } from './v9/toV10.js';
import { toV11 } from './v10/toV11.js';
import { toV12 } from './v11/toV12.js';
import { toV13 } from './v12/toV13.js';
import { toV14 } from './v13/toV14.js';
import { toV15 } from './v14/toV15.js';
import { toV16 } from './v15/toV16.js';
import { toLatest } from './v16/toLatest.js';
import { MagicNumber } from './MagicNumber.js';
import { LATEST_VERSION, TO_CALLS_VERSION } from './versions.js';
/**
 * @name MetadataVersioned
 * @description
 * The versioned runtime metadata as a decoded structure
 */
export class MetadataVersioned extends Struct {
    #converted = new Map();
    constructor(registry, value) {
        // const timeStart = performance.now()
        super(registry, {
            magicNumber: MagicNumber,
            metadata: 'MetadataAll'
        }, value);
        // console.log('MetadataVersioned', `${(performance.now() - timeStart).toFixed(2)}ms`)
    }
    #assertVersion = (version) => {
        if (this.version > version) {
            throw new Error(`Cannot convert metadata from version ${this.version} to ${version}`);
        }
        return this.version === version;
    };
    #getVersion = (version, fromPrev) => {
        if (version !== 'latest' && this.#assertVersion(version)) {
            const asCurr = `asV${version}`;
            return this.#metadata()[asCurr];
        }
        if (!this.#converted.has(version)) {
            const asPrev = version === 'latest'
                ? `asV${LATEST_VERSION}`
                : `asV${(version - 1)}`;
            this.#converted.set(version, fromPrev(this.registry, this[asPrev], this.version));
        }
        return this.#converted.get(version);
    };
    /**
     * @description the metadata wrapped
     */
    #metadata = () => {
        return this.getT('metadata');
    };
    /**
     * @description Returns the wrapped metadata as a limited calls-only (latest) version
     */
    get asCallsOnly() {
        return new MetadataVersioned(this.registry, {
            magicNumber: this.magicNumber,
            metadata: this.registry.createTypeUnsafe('MetadataAll', [toCallsOnly(this.registry, this.asLatest), TO_CALLS_VERSION])
        });
    }
    /**
     * @description Returns the wrapped metadata as a V9 object
     */
    get asV9() {
        this.#assertVersion(9);
        return this.#metadata().asV9;
    }
    /**
     * @description Returns the wrapped values as a V10 object
     */
    get asV10() {
        return this.#getVersion(10, toV10);
    }
    /**
     * @description Returns the wrapped values as a V11 object
     */
    get asV11() {
        return this.#getVersion(11, toV11);
    }
    /**
     * @description Returns the wrapped values as a V12 object
     */
    get asV12() {
        return this.#getVersion(12, toV12);
    }
    /**
     * @description Returns the wrapped values as a V13 object
     */
    get asV13() {
        return this.#getVersion(13, toV13);
    }
    /**
     * @description Returns the wrapped values as a V14 object
     */
    get asV14() {
        return this.#getVersion(14, toV14);
    }
    /**
     * @description Returns the wrapped values as a V15 object
     */
    get asV15() {
        return this.#getVersion(15, toV15);
    }
    /**
    * @description Returns the wrapped values as a V16 object
    */
    get asV16() {
        return this.#getVersion(16, toV16);
    }
    /**
     * @description Returns the wrapped values as a latest version object
     */
    get asLatest() {
        return this.#getVersion('latest', toLatest);
    }
    /**
     * @description The magicNumber for the Metadata (known constant)
     */
    get magicNumber() {
        return this.getT('magicNumber');
    }
    /**
     * @description the metadata version this structure represents
     */
    get version() {
        return this.#metadata().index;
    }
    getUniqTypes(throwError) {
        return getUniqTypes(this.registry, this.asLatest, throwError);
    }
    /**
     * @description Converts the Object to JSON, typically used for RPC transfers
     */
    toJSON() {
        // HACK(y): ensure that we apply the aliases if we have not done so already, this is
        // needed to ensure we have the correct overrides (which is only applied in toLatest)
        // eslint-disable-next-line no-unused-expressions
        this.asLatest;
        return super.toJSON();
    }
}
