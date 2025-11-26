"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/* eslint-disable sort-keys */
exports.default = {
    HrmpChannel: {
        maxCapacity: 'u32',
        maxTotalSize: 'u32',
        maxMessageSize: 'u32',
        msgCount: 'u32',
        totalSize: 'u32',
        mqcHead: 'Option<Hash>',
        senderDeposit: 'Balance',
        recipientDeposit: 'Balance'
    },
    HrmpChannelId: {
        sender: 'u32',
        receiver: 'u32'
    },
    HrmpOpenChannelRequest: {
        confirmed: 'bool',
        age: 'SessionIndex',
        senderDeposit: 'Balance',
        maxMessageSize: 'u32',
        maxCapacity: 'u32',
        maxTotalSize: 'u32'
    }
};
