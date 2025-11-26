import { Struct } from '@polkadot/types-codec';
import { UNMASK_VERSION } from './constants.js';
/**
 * @name GenericExtrinsicUnknown
 * @description
 * A default handler for extrinsics where the version is not known (default throw)
 */
export class GenericExtrinsicUnknown extends Struct {
    constructor(registry, _value, { isSigned = false, version = 0 } = {}) {
        super(registry, {});
        throw new Error(`Unsupported ${isSigned ? '' : 'un'}signed extrinsic version ${version & UNMASK_VERSION}`);
    }
}
