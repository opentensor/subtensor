"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const runtime_js_1 = require("./runtime.js");
const dmpQueue = {
    CollationInfo: {
        upwardMessages: 'Vec<UpwardMessage>',
        horizontalMessages: 'Vec<OutboundHrmpMessage>',
        newValidationCode: 'Option<ValidationCode>',
        processedDownwardMessages: 'u32',
        hrmpWatermark: 'RelayBlockNumber',
        headData: 'HeadData'
    },
    CollationInfoV1: {
        upwardMessages: 'Vec<UpwardMessage>',
        horizontalMessages: 'Vec<OutboundHrmpMessage>',
        newValidationCode: 'Option<ValidationCode>',
        processedDownwardMessages: 'u32',
        hrmpWatermark: 'RelayBlockNumber'
    },
    ConfigData: {
        maxIndividual: 'Weight'
    },
    MessageId: '[u8; 32]',
    OverweightIndex: 'u64',
    PageCounter: 'u32',
    PageIndexData: {
        beginUsed: 'PageCounter',
        endUsed: 'PageCounter',
        overweightCount: 'OverweightIndex'
    }
};
exports.default = {
    rpc: {},
    runtime: runtime_js_1.runtime,
    types: dmpQueue
};
