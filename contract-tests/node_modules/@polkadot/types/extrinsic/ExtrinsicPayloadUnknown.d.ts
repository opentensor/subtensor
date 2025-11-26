import type { Registry } from '@polkadot/types-codec/types';
import type { ExtrinsicPayloadOptions } from './types.js';
import { Struct } from '@polkadot/types-codec';
/**
 * @name GenericExtrinsicPayloadUnknown
 * @description
 * A default handler for payloads where the version is not known (default throw)
 */
export declare class GenericExtrinsicPayloadUnknown extends Struct {
    constructor(registry: Registry, _value?: unknown, { version }?: Partial<ExtrinsicPayloadOptions>);
}
