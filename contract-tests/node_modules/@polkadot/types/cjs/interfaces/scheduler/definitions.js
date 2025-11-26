"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = {
    rpc: {},
    types: {
        Period: '(BlockNumber, u32)',
        Priority: 'u8',
        SchedulePeriod: 'Period',
        SchedulePriority: 'Priority',
        Scheduled: {
            maybeId: 'Option<Bytes>',
            priority: 'SchedulePriority',
            call: 'Call',
            maybePeriodic: 'Option<SchedulePeriod>',
            origin: 'PalletsOrigin'
        },
        ScheduledTo254: {
            maybeId: 'Option<Bytes>',
            priority: 'SchedulePriority',
            call: 'Call',
            maybePeriodic: 'Option<SchedulePeriod>'
        },
        TaskAddress: '(BlockNumber, u32)'
    }
};
