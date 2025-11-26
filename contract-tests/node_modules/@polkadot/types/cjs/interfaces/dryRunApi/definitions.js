"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const runtime_js_1 = require("./runtime.js");
exports.default = {
    rpc: {},
    runtime: runtime_js_1.runtime,
    types: {
        PostDispatchInfo: {
            actualWeight: 'Option<Weight>',
            paysFee: 'Pays'
        },
        DispatchResultWithPostInfo: 'Result<PostDispatchInfo, DispatchError>',
        CallDryRunEffects: {
            executionResult: 'DispatchResultWithPostInfo',
            emittedEvents: 'Vec<Event>',
            localXcm: 'Option<VersionedXcm>',
            forwardedXcms: 'Vec<(VersionedMultiLocation, Vec<VersionedXcm>)>'
        },
        XcmDryRunEffects: {
            executionResult: 'OutcomeV4',
            emittedEvents: 'Vec<Event>',
            forwardedXcms: 'Vec<(VersionedMultiLocation, Vec<VersionedXcm>)>'
        },
        XcmDryRunApiError: {
            _enum: [
                'Unimplemented',
                'VersionedConversionFailed'
            ]
        }
    }
};
