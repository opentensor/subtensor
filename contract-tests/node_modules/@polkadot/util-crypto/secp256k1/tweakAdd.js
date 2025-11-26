import { _0n, BN, bnToU8a, hasBigInt, isU8a, nToU8a, u8aToBigInt } from '@polkadot/util';
import { BigInt } from '@polkadot/x-bigint';
import { BN_BE_256_OPTS, BN_BE_OPTS } from '../bn.js';
const N = 'ffffffff ffffffff ffffffff fffffffe baaedce6 af48a03b bfd25e8c d0364141'.replace(/ /g, '');
const N_BI = BigInt(`0x${N}`);
const N_BN = new BN(N, 'hex');
function addBi(seckey, tweak) {
    let res = u8aToBigInt(tweak, BN_BE_OPTS);
    if (res >= N_BI) {
        throw new Error('Tweak parameter is out of range');
    }
    res += u8aToBigInt(seckey, BN_BE_OPTS);
    if (res >= N_BI) {
        res -= N_BI;
    }
    if (res === _0n) {
        throw new Error('Invalid resulting private key');
    }
    return nToU8a(res, BN_BE_256_OPTS);
}
function addBn(seckey, tweak) {
    const res = new BN(tweak);
    if (res.cmp(N_BN) >= 0) {
        throw new Error('Tweak parameter is out of range');
    }
    res.iadd(new BN(seckey));
    if (res.cmp(N_BN) >= 0) {
        res.isub(N_BN);
    }
    if (res.isZero()) {
        throw new Error('Invalid resulting private key');
    }
    return bnToU8a(res, BN_BE_256_OPTS);
}
export function secp256k1PrivateKeyTweakAdd(seckey, tweak, onlyBn) {
    if (!isU8a(seckey) || seckey.length !== 32) {
        throw new Error('Expected seckey to be an Uint8Array with length 32');
    }
    else if (!isU8a(tweak) || tweak.length !== 32) {
        throw new Error('Expected tweak to be an Uint8Array with length 32');
    }
    return !hasBigInt || onlyBn
        ? addBn(seckey, tweak)
        : addBi(seckey, tweak);
}
