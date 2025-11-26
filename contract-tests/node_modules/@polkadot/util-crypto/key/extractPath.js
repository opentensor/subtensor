import { DeriveJunction } from './DeriveJunction.js';
const RE_JUNCTION = /\/(\/?)([^/]+)/g;
/**
 * @description Extract derivation junctions from the supplied path
 */
export function keyExtractPath(derivePath) {
    const parts = derivePath.match(RE_JUNCTION);
    const path = [];
    let constructed = '';
    if (parts) {
        constructed = parts.join('');
        for (const p of parts) {
            path.push(DeriveJunction.from(p.substring(1)));
        }
    }
    if (constructed !== derivePath) {
        throw new Error(`Re-constructed path "${constructed}" does not match input`);
    }
    return {
        parts,
        path
    };
}
