import { Struct } from '@polkadot/types-codec';
/**
 * @name GenericExtrinsicPayloadUnknown
 * @description
 * A default handler for payloads where the version is not known (default throw)
 */
export class GenericExtrinsicPayloadUnknown extends Struct {
    constructor(registry, _value, { version = 0 } = {}) {
        super(registry, {});
        throw new Error(`Unsupported extrinsic payload version ${version}`);
    }
}
