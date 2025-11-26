import { u8aToHex, u8aToU8a } from '@polkadot/util';
import { keccakAsU8a } from '../keccak/index.js';
import { secp256k1Expand } from '../secp256k1/index.js';
function getH160(u8a) {
    if ([33, 65].includes(u8a.length)) {
        u8a = keccakAsU8a(secp256k1Expand(u8a));
    }
    return u8a.slice(-20);
}
export function ethereumEncode(addressOrPublic) {
    if (!addressOrPublic) {
        return '0x';
    }
    const u8aAddress = u8aToU8a(addressOrPublic);
    if (![20, 32, 33, 65].includes(u8aAddress.length)) {
        throw new Error(`Invalid address or publicKey provided, received ${u8aAddress.length} bytes input`);
    }
    const address = u8aToHex(getH160(u8aAddress), -1, false);
    const hash = u8aToHex(keccakAsU8a(address), -1, false);
    let result = '';
    for (let i = 0; i < 40; i++) {
        result = `${result}${parseInt(hash[i], 16) > 7 ? address[i].toUpperCase() : address[i]}`;
    }
    return `0x${result}`;
}
