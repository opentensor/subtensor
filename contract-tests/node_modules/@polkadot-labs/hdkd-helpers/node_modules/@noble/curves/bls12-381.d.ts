import { type BlsCurvePairWithSignatures } from './abstract/bls.ts';
import { type IField } from './abstract/modular.ts';
/** bls12-381 Fr (Fn) field. Note: does mod() on fromBytes, due to modFromBytes option. */
export declare const bls12_381_Fr: IField<bigint>;
/**
 * bls12-381 pairing-friendly curve construction.
 * Provides both longSignatures and shortSignatures.
 */
export declare const bls12_381: BlsCurvePairWithSignatures;
//# sourceMappingURL=bls12-381.d.ts.map