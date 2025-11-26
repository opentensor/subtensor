import { sr25519DeriveKeypairSoft } from '@polkadot/wasm-crypto';
import { createDeriveFn } from './derive.js';
export const sr25519DeriveSoft = /*#__PURE__*/ createDeriveFn(sr25519DeriveKeypairSoft);
