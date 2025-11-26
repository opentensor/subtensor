import { isU8a } from '@polkadot/util';
import { sr25519PairFromU8a } from './pair/fromU8a.js';
import { sr25519KeypairToU8a } from './pair/toU8a.js';
export function createDeriveFn(derive) {
    return (keypair, chainCode) => {
        if (!isU8a(chainCode) || chainCode.length !== 32) {
            throw new Error('Invalid chainCode passed to derive');
        }
        return sr25519PairFromU8a(derive(sr25519KeypairToU8a(keypair), chainCode));
    };
}
