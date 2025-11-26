"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GenericExtrinsicUnknown = void 0;
const types_codec_1 = require("@polkadot/types-codec");
const constants_js_1 = require("./constants.js");
/**
 * @name GenericExtrinsicUnknown
 * @description
 * A default handler for extrinsics where the version is not known (default throw)
 */
class GenericExtrinsicUnknown extends types_codec_1.Struct {
    constructor(registry, _value, { isSigned = false, version = 0 } = {}) {
        super(registry, {});
        throw new Error(`Unsupported ${isSigned ? '' : 'un'}signed extrinsic version ${version & constants_js_1.UNMASK_VERSION}`);
    }
}
exports.GenericExtrinsicUnknown = GenericExtrinsicUnknown;
