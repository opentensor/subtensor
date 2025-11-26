"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GenericExtrinsicPayloadUnknown = void 0;
const types_codec_1 = require("@polkadot/types-codec");
/**
 * @name GenericExtrinsicPayloadUnknown
 * @description
 * A default handler for payloads where the version is not known (default throw)
 */
class GenericExtrinsicPayloadUnknown extends types_codec_1.Struct {
    constructor(registry, _value, { version = 0 } = {}) {
        super(registry, {});
        throw new Error(`Unsupported extrinsic payload version ${version}`);
    }
}
exports.GenericExtrinsicPayloadUnknown = GenericExtrinsicPayloadUnknown;
