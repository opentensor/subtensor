import type { Keypair, KeypairType } from '../types.js';
import type { DeriveJunction } from './DeriveJunction.js';
export declare function keyFromPath(pair: Keypair, path: DeriveJunction[], type: KeypairType): Keypair;
