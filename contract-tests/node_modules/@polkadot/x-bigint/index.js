import { extractGlobal } from '@polkadot/x-global';
export { packageInfo } from './packageInfo.js';
/**
 * @internal
 *
 * There are _still_ some older environments (specifically RN < 0.70), that does
 * not have proper BigInt support - a non-working fallback is provided for those.
 *
 * We detect availability of BigInt upon usage, so this is purely to allow functional
 * compilation & bundling. Since we have operators such as *+-/ top-level, a number-ish
 * result is used here.
 */
function invalidFallback() {
    return Number.NaN;
}
export const BigInt = /*#__PURE__*/ extractGlobal('BigInt', invalidFallback);
