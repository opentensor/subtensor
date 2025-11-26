import { u8aConcat } from '@polkadot/util';
export function sr25519KeypairToU8a({ publicKey, secretKey }) {
    return u8aConcat(secretKey, publicKey).slice();
}
