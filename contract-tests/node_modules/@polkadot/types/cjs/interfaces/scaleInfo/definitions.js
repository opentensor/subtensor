"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const v0_js_1 = require("./v0.js");
const v1_js_1 = require("./v1.js");
/* eslint-disable sort-keys */
exports.default = {
    rpc: {},
    types: {
        ...v0_js_1.v0,
        ...v1_js_1.v1,
        // latest mappings
        SiField: 'Si1Field',
        SiLookupTypeId: 'Si1LookupTypeId',
        SiPath: 'Si1Path',
        SiType: 'Si1Type',
        SiTypeDef: 'Si1TypeDef',
        SiTypeDefArray: 'Si1TypeDefArray',
        SiTypeDefBitSequence: 'Si1TypeDefBitSequence',
        SiTypeDefCompact: 'Si1TypeDefCompact',
        SiTypeDefComposite: 'Si1TypeDefComposite',
        SiTypeDefPrimitive: 'Si1TypeDefPrimitive',
        SiTypeDefSequence: 'Si1TypeDefSequence',
        SiTypeDefTuple: 'Si1TypeDefTuple',
        SiTypeParameter: 'Si1TypeParameter',
        SiTypeDefVariant: 'Si1TypeDefVariant',
        SiVariant: 'Si1Variant'
    }
};
