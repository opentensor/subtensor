import { Option, Struct } from '@polkadot/types-codec';
import { objectProperty, objectSpread, u8aToHex } from '@polkadot/util';
const knownTypes = {
    address: 'Address',
    assetId: 'Option<TAssetConversion>',
    blockHash: 'Hash',
    blockNumber: 'BlockNumber',
    era: 'ExtrinsicEra',
    genesisHash: 'Hash',
    metadataHash: 'Option<[u8;32]>',
    method: 'Call',
    mode: 'u8',
    nonce: 'Compact<Index>',
    runtimeVersion: 'RuntimeVersion',
    signedExtensions: 'Vec<Text>',
    tip: 'Compact<Balance>',
    version: 'u8'
};
/**
 * @name GenericSignerPayload
 * @description
 * A generic signer payload that can be used for serialization between API and signer
 */
export class GenericSignerPayload extends Struct {
    #extraTypes;
    constructor(registry, value) {
        const extensionTypes = objectSpread({}, registry.getSignedExtensionTypes(), registry.getSignedExtensionExtra());
        super(registry, objectSpread({}, extensionTypes, knownTypes, { withSignedTransaction: 'bool' }), value);
        this.#extraTypes = {};
        const getter = (key) => this.get(key);
        // add all extras that are not in the base types
        for (const [key, type] of Object.entries(extensionTypes)) {
            if (!knownTypes[key]) {
                this.#extraTypes[key] = type;
            }
            objectProperty(this, key, getter);
        }
    }
    get address() {
        return this.getT('address');
    }
    get blockHash() {
        return this.getT('blockHash');
    }
    get blockNumber() {
        return this.getT('blockNumber');
    }
    get era() {
        return this.getT('era');
    }
    get genesisHash() {
        return this.getT('genesisHash');
    }
    get method() {
        return this.getT('method');
    }
    get nonce() {
        return this.getT('nonce');
    }
    get runtimeVersion() {
        return this.getT('runtimeVersion');
    }
    get signedExtensions() {
        return this.getT('signedExtensions');
    }
    get tip() {
        return this.getT('tip');
    }
    get assetId() {
        return this.getT('assetId');
    }
    get version() {
        return this.getT('version');
    }
    get mode() {
        return this.getT('mode');
    }
    get metadataHash() {
        return this.getT('metadataHash');
    }
    get withSignedTransaction() {
        const val = this.getT('withSignedTransaction');
        return val.isTrue;
    }
    /**
     * @description Creates an representation of the structure as an ISignerPayload JSON
     */
    toPayload() {
        const result = {};
        const keys = Object.keys(this.#extraTypes);
        // add any explicit overrides we may have
        for (let i = 0, count = keys.length; i < count; i++) {
            const key = keys[i];
            const value = this.getT(key);
            // Don't include Option.isNone
            if (!(value instanceof Option) || value.isSome) {
                // NOTE In the spread below we convert (mostly) to Hex to align
                // with the typings. In the case of "unknown" fields, we use the
                // primitive toJSON conversion (which is serializable). Technically
                // we can include isNone in here as well ("null" is allowed), however
                // for empty fields we just skip it completely (historical compat)
                result[key] = value.toJSON();
            }
        }
        return objectSpread(result, {
            // the known defaults as managed explicitly and has different
            // formatting in cases, e.g. we mostly expose a hex format here
            address: this.address.toString(),
            assetId: this.assetId && this.assetId.isSome ? this.assetId.toHex() : null,
            blockHash: this.blockHash.toHex(),
            blockNumber: this.blockNumber.toHex(),
            era: this.era.toHex(),
            genesisHash: this.genesisHash.toHex(),
            metadataHash: this.metadataHash.isSome ? this.metadataHash.toHex() : null,
            method: this.method.toHex(),
            mode: this.mode.toNumber(),
            nonce: this.nonce.toHex(),
            signedExtensions: this.signedExtensions.map((e) => e.toString()),
            specVersion: this.runtimeVersion.specVersion.toHex(),
            tip: this.tip.toHex(),
            transactionVersion: this.runtimeVersion.transactionVersion.toHex(),
            version: this.version.toNumber(),
            withSignedTransaction: this.withSignedTransaction
        });
    }
    /**
     * @description Creates a representation of the payload in raw Exrinsic form
     */
    toRaw() {
        const payload = this.toPayload();
        const data = u8aToHex(this.registry
            .createTypeUnsafe('ExtrinsicPayload', [payload, { version: payload.version }])
            // NOTE Explicitly pass the bare flag so the method is encoded un-prefixed (non-decodable, for signing only)
            .toU8a({ method: true }));
        return {
            address: payload.address,
            data,
            type: 'payload'
        };
    }
}
