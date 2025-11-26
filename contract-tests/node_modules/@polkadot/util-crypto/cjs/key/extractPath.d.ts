import { DeriveJunction } from './DeriveJunction.js';
export interface ExtractResult {
    parts: string[] | null;
    path: DeriveJunction[];
}
/**
 * @description Extract derivation junctions from the supplied path
 */
export declare function keyExtractPath(derivePath: string): ExtractResult;
