export default {
    rpc: {},
    types: {
        DeferredOffenceOf: '(Vec<OffenceDetails>, Vec<Perbill>, SessionIndex)',
        Kind: '[u8; 16]',
        OffenceDetails: {
            offender: 'Offender',
            reporters: 'Vec<Reporter>'
        },
        Offender: 'IdentificationTuple',
        OpaqueTimeSlot: 'Bytes',
        ReportIdOf: 'Hash',
        Reporter: 'AccountId'
    }
};
