import type { Enum } from '@polkadot/types-codec';
/** @name XcmPaymentApiError */
export interface XcmPaymentApiError extends Enum {
    readonly isUnimplemented: boolean;
    readonly isVersionedConversionFailed: boolean;
    readonly isWeightNotComputable: boolean;
    readonly isUnhandledXcmVersion: boolean;
    readonly isAssetNotFound: boolean;
    readonly type: 'Unimplemented' | 'VersionedConversionFailed' | 'WeightNotComputable' | 'UnhandledXcmVersion' | 'AssetNotFound';
}
export type PHANTOM_XCMPAYMENTAPI = 'xcmPaymentApi';
